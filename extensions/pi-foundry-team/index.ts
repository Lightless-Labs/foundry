import { spawn } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";

type JsonObject = Record<string, unknown>;

type FoundryAgent = {
	name: string;
	description: string;
	tools: string[];
	model?: string;
	systemPrompt: string;
	filePath: string;
};

type DispatchRequest = {
	envelopePath: string;
	agent?: string;
	model?: string;
	tools?: string[];
	cwd?: string;
	includeContextFiles?: boolean;
};

type DispatchResult = {
	envelopePath: string;
	recipient: string;
	phase: string;
	agent?: string;
	plannedModel?: string;
	actualModel?: string;
	exitCode: number;
	stopReason?: string;
	errorMessage?: string;
	output: string;
	stderr: string;
};

const PACKAGE_ROOT = path.resolve(__dirname, "../..");
const AGENTS_ROOT = path.join(PACKAGE_ROOT, "plugins", "foundry", "agents");
const MAX_PARALLEL_DISPATCHES = 8;
const MAX_CONCURRENCY = 4;
const OUTPUT_CAP_BYTES = 50 * 1024;
const ARBITER_REQUIRED_VISIBLE_CONTEXT: Array<[string, RegExp]> = [
	["spec_or_nlspec", /(?:^|[_ -])(nl)?spec(?:$|[_ -])/i],
	["single_disputed_test", /disputed[_ -]?test|test[_ -]?artifact/i],
	["implementation_snippet", /implementation|relevant[_ -]?snippet/i],
	["runner_result", /runner[_ -]?result|test[_ -]?result|raw[_ -]?output|outcome/i],
];
const ARBITER_OVERBROAD_VISIBLE_CONTEXT = [
	/full[_ -]?test[_ -]?suite|all[_ -]?tests|complete[_ -]?test[_ -]?suite/i,
	/full[_ -]?implementation|complete[_ -]?implementation|whole[_ -]?implementation|implementation[_ -]?tree/i,
	/conversation[_ -]?history|red[_ -]?green[_ -]?history|chat[_ -]?history|transcript/i,
];

function parseFrontmatter(markdown: string): { frontmatter: Record<string, string>; body: string } {
	if (!markdown.startsWith("---\n")) return { frontmatter: {}, body: markdown };
	const end = markdown.indexOf("\n---", 4);
	if (end === -1) return { frontmatter: {}, body: markdown };
	const raw = markdown.slice(4, end).trim();
	const body = markdown.slice(end + "\n---".length).replace(/^\n/, "");
	const frontmatter: Record<string, string> = {};
	for (const line of raw.split("\n")) {
		const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
		if (!match) continue;
		frontmatter[match[1]] = match[2].replace(/^['\"]|['\"]$/g, "").trim();
	}
	return { frontmatter, body };
}

function normalizeTools(raw: string | undefined): string[] {
	if (!raw) return [];
	const map: Record<string, string> = {
		Read: "read",
		Grep: "grep",
		Glob: "find",
		Bash: "bash",
		Write: "write",
		Edit: "edit",
		Ls: "ls",
		LS: "ls",
	};
	return raw
		.split(",")
		.map((tool) => tool.trim())
		.filter(Boolean)
		.map((tool) => map[tool] ?? tool.toLowerCase())
		.filter((tool, index, all) => all.indexOf(tool) === index);
}

function discoverFoundryAgents(): FoundryAgent[] {
	const agents: FoundryAgent[] = [];
	const walk = (dir: string) => {
		if (!fs.existsSync(dir)) return;
		for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
			const filePath = path.join(dir, entry.name);
			if (entry.isDirectory()) {
				walk(filePath);
				continue;
			}
			if (!entry.name.endsWith(".md")) continue;
			const content = fs.readFileSync(filePath, "utf8");
			const { frontmatter, body } = parseFrontmatter(content);
			if (!frontmatter.name || !frontmatter.description) continue;
			agents.push({
				name: frontmatter.name,
				description: frontmatter.description,
				tools: normalizeTools(frontmatter.tools),
				model: frontmatter.model && frontmatter.model !== "inherit" ? frontmatter.model : undefined,
				systemPrompt: body,
				filePath,
			});
		}
	};
	walk(AGENTS_ROOT);
	return agents.sort((a, b) => a.name.localeCompare(b.name));
}

function agentNameSuffix(name: string): string {
	return name.trim().split(":").filter(Boolean).pop() ?? name.trim();
}

function resolveFoundryAgent(requested: string | undefined, agents: FoundryAgent[]): FoundryAgent | undefined {
	if (!requested) return undefined;
	const normalized = requested.trim();
	if (!normalized) return undefined;
	const lower = normalized.toLowerCase();
	const exact = agents.find((candidate) => candidate.name === normalized || candidate.name.toLowerCase() === lower);
	if (exact) return exact;

	const suffix = agentNameSuffix(normalized).toLowerCase();
	const suffixMatches = agents.filter((candidate) => agentNameSuffix(candidate.name).toLowerCase() === suffix);
	return suffixMatches.length === 1 ? suffixMatches[0] : undefined;
}

function availableFoundryAgentNames(agents: FoundryAgent[]): string {
	return agents.map((agent) => agent.name).join(", ") || "none";
}

function resolvePath(cwd: string, rawPath: string): string {
	const expanded = rawPath.startsWith("~/") ? path.join(os.homedir(), rawPath.slice(2)) : rawPath;
	return path.isAbsolute(expanded) ? expanded : path.resolve(cwd, expanded);
}

function readEnvelope(cwd: string, envelopePath: string): JsonObject {
	const absolutePath = resolvePath(cwd, envelopePath);
	const data = JSON.parse(fs.readFileSync(absolutePath, "utf8"));
	if (!data || typeof data !== "object" || Array.isArray(data)) throw new Error("Envelope must be a JSON object");
	return data as JsonObject;
}

function samplesFrom(withheld: unknown): string[] {
	if (!Array.isArray(withheld)) return [];
	const samples: string[] = [];
	for (const item of withheld) {
		if (!item || typeof item !== "object" || Array.isArray(item)) continue;
		const rawSamples = (item as JsonObject).samples;
		if (!Array.isArray(rawSamples)) continue;
		for (const sample of rawSamples) {
			if (typeof sample === "string" && sample.trim().length >= 8) samples.push(sample.trim());
		}
	}
	return samples;
}

function testResultLabelNames(prompt: string): Set<string> {
	const labels = new Set<string>();
	let inResults = false;
	for (const line of prompt.split(/\r?\n/)) {
		const stripped = line.trim();
		if (stripped === "Test results:") {
			inResults = true;
			continue;
		}
		if (!inResults) continue;
		if (!stripped) continue;
		if (stripped.startsWith("## ") || stripped.startsWith("# ")) {
			inResults = false;
			continue;
		}
		const match = stripped.match(/^(.+?):\s*(PASS|FAIL)\s*$/);
		if (!match) continue;
		const label = match[1].trim();
		labels.add(label);
		for (const separator of ["::", "/"]) {
			if (label.includes(separator)) labels.add(label.split(separator).pop() ?? label);
		}
	}
	return labels;
}

function sampleIsOutcomeLabel(sample: string, labels: Set<string>): boolean {
	if (labels.has(sample)) return true;
	for (const label of labels) {
		if (sample.length >= 8 && (label.includes(sample) || sample.includes(label))) return true;
	}
	return false;
}

function contextName(item: unknown): string {
	if (!item || typeof item !== "object" || Array.isArray(item)) return "";
	const context = item as JsonObject;
	return `${typeof context.label === "string" ? context.label : ""} ${typeof context.kind === "string" ? context.kind : ""}`;
}

function validateArbiterScope(envelope: JsonObject, prompt: string): void {
	if (!prompt.includes("ArbiterInput:")) throw new Error("Arbiter envelope prompt must contain an ArbiterInput packet");
	if (/\bdisputed_tests\s*:/.test(prompt)) {
		throw new Error("Arbiter envelope must contain exactly one disputed_test, not disputed_tests");
	}
	if ((prompt.match(/^\s*disputed_test\s*:/gm) ?? []).length !== 1) {
		throw new Error("Arbiter envelope must contain exactly one disputed_test block");
	}
	if ((prompt.match(/^\s*test_artifact\s*:/gm) ?? []).length !== 1) {
		throw new Error("Arbiter envelope must contain exactly one test_artifact block");
	}

	const visible = envelope.visible_context;
	if (!Array.isArray(visible)) throw new Error("Envelope visible_context must be a list");
	for (let i = 0; i < visible.length; i++) {
		const item = visible[i];
		if (!item || typeof item !== "object" || Array.isArray(item)) {
			throw new Error(`visible_context[${i}] must be an object`);
		}
		const name = contextName(item);
		for (const pattern of ARBITER_OVERBROAD_VISIBLE_CONTEXT) {
			if (pattern.test(name)) throw new Error(`Arbiter visible_context is over-broad: ${name}`);
		}
	}

	const missing = ARBITER_REQUIRED_VISIBLE_CONTEXT
		.filter(([, pattern]) => !visible.some((item) => pattern.test(contextName(item))))
		.map(([label]) => label);
	if (missing.length > 0) throw new Error(`Arbiter visible_context missing scoped context: ${missing.join(", ")}`);

	const redactionsText = JSON.stringify(envelope.redactions ?? []);
	if (!redactionsText.includes("single_test_scope")) {
		throw new Error("Arbiter envelope redactions must include single_test_scope");
	}
	if (samplesFrom(envelope.withheld_context).length === 0) {
		throw new Error("Arbiter envelope must include at least one meaningful withheld_context sample");
	}
}

function validateEnvelope(envelope: JsonObject): { prompt: string; recipient: string; phase: string } {
	if (envelope.schema_version !== "foundry.prompt-envelope.v1") {
		throw new Error("Envelope schema_version must be foundry.prompt-envelope.v1");
	}
	const prompt = envelope.prompt;
	if (typeof prompt !== "string" || prompt.trim() === "") throw new Error("Envelope prompt must be a non-empty string");
	const recipient = typeof envelope.recipient === "string" ? envelope.recipient : "unknown";
	const phase = typeof envelope.phase === "string" ? envelope.phase : "unknown";
	if (!Array.isArray(envelope.visible_context)) throw new Error("Envelope visible_context must be a list");
	if (!Array.isArray(envelope.withheld_context)) throw new Error("Envelope withheld_context must be a list");

	const lowerRecipient = recipient.toLowerCase();
	const isRedOrGreen = /(^|[-_])(?:red|green)([-_]|$)|red-team|green-team|red-reviewer|green-reviewer/.test(
		lowerRecipient,
	);
	const isArbiter = /(^|[:/_-])arbiter-agent($|[:/_-])|foundry:review:arbiter-agent/i.test(recipient);
	const samples = samplesFrom(envelope.withheld_context);
	if (isRedOrGreen && samples.length === 0) {
		throw new Error("Red/green recipient envelopes must include meaningful withheld_context samples");
	}
	if (isArbiter) validateArbiterScope(envelope, prompt);
	const outcomeLabels = lowerRecipient.includes("green") ? testResultLabelNames(prompt) : new Set<string>();
	for (const sample of samples) {
		if (outcomeLabels.size > 0 && sampleIsOutcomeLabel(sample, outcomeLabels)) {
			throw new Error(`Withheld sample duplicates allowed PASS/FAIL outcome label: ${sample.slice(0, 80)}`);
		}
		if (prompt.includes(sample)) {
			throw new Error(`Withheld sample leaked into prompt: ${sample.slice(0, 80)}`);
		}
	}
	return { prompt, recipient, phase };
}

function finalAssistantText(messages: any[]): string {
	for (let i = messages.length - 1; i >= 0; i--) {
		const msg = messages[i];
		if (msg?.role !== "assistant" || !Array.isArray(msg.content)) continue;
		for (const part of msg.content) {
			if (part?.type === "text" && typeof part.text === "string") return part.text;
		}
	}
	return "";
}

function modelLaneId(message: any): string | undefined {
	const model = typeof message?.model === "string" && message.model.trim() ? message.model.trim() : undefined;
	const provider = typeof message?.provider === "string" && message.provider.trim() ? message.provider.trim() : undefined;
	if (!model) return undefined;
	if (!provider || model.includes("/")) return model;
	return `${provider}/${model}`;
}

function splitThinkingLane(model: string | undefined): { base: string; thinking?: string } | undefined {
	if (!model) return undefined;
	const match = model.match(/^(.*):(off|minimal|low|medium|high|xhigh)$/);
	if (!match) return { base: model };
	return { base: match[1], thinking: match[2] };
}

function actualModelLane(actualModel: string | undefined, plannedModel: string | undefined): string | undefined {
	if (!actualModel) return undefined;
	const actual = splitThinkingLane(actualModel);
	const planned = splitThinkingLane(plannedModel);
	if (actual && planned?.thinking && actual.base === planned.base && !actual.thinking) {
		return `${actual.base}:${planned.thinking}`;
	}
	return actualModel;
}

function truncateOutput(text: string): string {
	const bytes = Buffer.byteLength(text, "utf8");
	if (bytes <= OUTPUT_CAP_BYTES) return text;
	let truncated = text.slice(0, OUTPUT_CAP_BYTES);
	while (Buffer.byteLength(truncated, "utf8") > OUTPUT_CAP_BYTES) truncated = truncated.slice(0, -1);
	return `${truncated}\n\n[Output truncated: ${bytes - Buffer.byteLength(truncated, "utf8")} bytes omitted.]`;
}

async function writeTempPrompt(prefix: string, content: string): Promise<{ dir: string; filePath: string }> {
	const dir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "foundry-pi-agent-"));
	const filePath = path.join(dir, `${prefix}.md`);
	await fs.promises.writeFile(filePath, content, { encoding: "utf8", mode: 0o600 });
	return { dir, filePath };
}

function piInvocation(args: string[]): { command: string; args: string[] } {
	const currentScript = process.argv[1];
	const isBunVirtualScript = currentScript?.startsWith("/$bunfs/root/");
	if (currentScript && !isBunVirtualScript && fs.existsSync(currentScript)) {
		return { command: process.execPath, args: [currentScript, ...args] };
	}
	return { command: "pi", args };
}

async function runDispatch(
	ctxCwd: string,
	agents: FoundryAgent[],
	request: DispatchRequest,
	signal: AbortSignal | undefined,
): Promise<DispatchResult> {
	const envelope = readEnvelope(ctxCwd, request.envelopePath);
	const { prompt, recipient, phase } = validateEnvelope(envelope);
	const agent = request.agent ? resolveFoundryAgent(request.agent, agents) : resolveFoundryAgent(recipient, agents);
	if (request.agent && !agent) {
		throw new Error(
			`Unknown Foundry agent ${request.agent}. Use a packaged frontmatter name or omit agent to infer from envelope.recipient. Available agents: ${availableFoundryAgentNames(agents)}`,
		);
	}

	const plannedModel = request.model ?? agent?.model;
	const tools = request.tools ?? agent?.tools ?? ["read", "grep", "find", "ls"];
	const args = ["--mode", "json", "-p", "--no-session", "--no-extensions", "--no-skills", "--no-prompt-templates"];
	if (!request.includeContextFiles) args.push("--no-context-files");
	if (plannedModel) args.push("--model", plannedModel);
	if (tools.length > 0) args.push("--tools", tools.join(","));

	let temp: { dir: string; filePath: string } | undefined;
	try {
		if (agent?.systemPrompt.trim()) {
			temp = await writeTempPrompt(agent.name.replace(/[^A-Za-z0-9_.-]/g, "-"), agent.systemPrompt);
			args.push("--append-system-prompt", temp.filePath);
		}
		args.push(prompt);

		const messages: any[] = [];
		let stderr = "";
		let actualModel: string | undefined;
		let stopReason: string | undefined;
		let errorMessage: string | undefined;
		const cwd = request.cwd ? resolvePath(ctxCwd, request.cwd) : ctxCwd;
		const invocation = piInvocation(args);
		let wasAborted = false;

		const exitCode = await new Promise<number>((resolve) => {
			const child = spawn(invocation.command, invocation.args, {
				cwd,
				shell: false,
				stdio: ["ignore", "pipe", "pipe"],
			});
			let buffer = "";
			const processLine = (line: string) => {
				if (!line.trim()) return;
				let event: any;
				try {
					event = JSON.parse(line);
				} catch {
					return;
				}
				if (event.type === "message_end" && event.message) {
					messages.push(event.message);
					if (event.message.role === "assistant") {
						const laneId = modelLaneId(event.message);
						if (laneId) actualModel = laneId;
						if (event.message.stopReason) stopReason = event.message.stopReason;
						if (event.message.errorMessage) errorMessage = event.message.errorMessage;
					}
				}
				if (event.type === "tool_result_end" && event.message) messages.push(event.message);
			};
			child.stdout.on("data", (data) => {
				buffer += data.toString();
				const lines = buffer.split("\n");
				buffer = lines.pop() ?? "";
				for (const line of lines) processLine(line);
			});
			child.stderr.on("data", (data) => {
				stderr += data.toString();
			});
			child.on("close", (code) => {
				if (buffer.trim()) processLine(buffer);
				resolve(code ?? 0);
			});
			child.on("error", () => resolve(1));
			if (signal) {
				const kill = () => {
					wasAborted = true;
					child.kill("SIGTERM");
					setTimeout(() => child.kill("SIGKILL"), 5000);
				};
				if (signal.aborted) kill();
				else signal.addEventListener("abort", kill, { once: true });
			}
		});
		if (wasAborted) throw new Error("Foundry child dispatch aborted");
		return {
			envelopePath: request.envelopePath,
			recipient,
			phase,
			agent: agent?.name,
			plannedModel,
			actualModel: actualModelLane(actualModel, plannedModel),
			exitCode,
			stopReason,
			errorMessage,
			output: truncateOutput(errorMessage || finalAssistantText(messages) || "(no output)"),
			stderr: truncateOutput(stderr),
		};
	} finally {
		if (temp) {
			try {
				fs.unlinkSync(temp.filePath);
			} catch {
				/* ignore */
			}
			try {
				fs.rmdirSync(temp.dir);
			} catch {
				/* ignore */
			}
		}
	}
}

async function mapWithConcurrency<T>(items: DispatchRequest[], concurrency: number, fn: (item: DispatchRequest) => Promise<T>) {
	const results: T[] = new Array(items.length);
	let next = 0;
	const workers = new Array(Math.min(concurrency, items.length)).fill(null).map(async () => {
		while (next < items.length) {
			const index = next++;
			results[index] = await fn(items[index]);
		}
	});
	await Promise.all(workers);
	return results;
}

const DispatchItem = Type.Object({
	envelopePath: Type.String({ description: "Path to a foundry.prompt-envelope.v1 JSON artifact" }),
	agent: Type.Optional(Type.String({ description: "Optional Foundry agent name for system prompt selection" })),
	model: Type.Optional(Type.String({ description: "Optional pi model id/pattern for this child dispatch" })),
	tools: Type.Optional(Type.Array(Type.String(), { description: "Optional pi tool allowlist for the child dispatch" })),
	cwd: Type.Optional(Type.String({ description: "Optional child working directory" })),
	includeContextFiles: Type.Optional(Type.Boolean({ description: "Allow child pi to load AGENTS.md/CLAUDE.md context files. Default false." })),
});

const FoundryTeamParams = Type.Object({
	envelopePath: Type.Optional(Type.String({ description: "Single-dispatch PromptEnvelope path" })),
	agent: Type.Optional(Type.String({ description: "Optional Foundry agent name for single dispatch" })),
	model: Type.Optional(Type.String({ description: "Optional pi model id/pattern for single dispatch" })),
	tools: Type.Optional(Type.Array(Type.String(), { description: "Optional pi tool allowlist for single dispatch" })),
	cwd: Type.Optional(Type.String({ description: "Optional child working directory for single dispatch" })),
	includeContextFiles: Type.Optional(Type.Boolean({ description: "Allow child pi to load AGENTS.md/CLAUDE.md context files. Default false." })),
	dispatches: Type.Optional(Type.Array(DispatchItem, { description: "Parallel dispatches. Max 8, executed with concurrency 4." })),
	agentScope: Type.Optional(StringEnum(["foundry-package"] as const, { default: "foundry-package" })),
});

export default function (pi: ExtensionAPI) {
	pi.registerTool({
		name: "foundry_team",
		label: "Foundry Team",
		description: [
			"Dispatch isolated Foundry child agents from PromptEnvelope JSON artifacts.",
			"Use this instead of non-existent built-in subagents when running Foundry under pi.",
			"The tool validates withheld samples before spawning child pi processes and sends exactly envelope.prompt.",
			"Supports single dispatch (envelopePath) and bounded parallel dispatch (dispatches array).",
		].join(" "),
		promptSnippet: "Dispatch Foundry red/green/reviewer child agents from PromptEnvelope artifacts with isolated pi contexts.",
		promptGuidelines: [
			"Use foundry_team for Foundry subagent/team dispatches under pi; pi has no built-in Agent(...) primitive.",
			"Before calling foundry_team, write and validate a PromptEnvelope JSON artifact; pass its path as envelopePath and do not paste hidden context into the task.",
		],
		parameters: FoundryTeamParams,

		async execute(_toolCallId, params, signal, onUpdate, ctx) {
			const agents = discoverFoundryAgents();
			const hasSingle = Boolean(params.envelopePath);
			const hasParallel = Boolean(params.dispatches && params.dispatches.length > 0);
			if (Number(hasSingle) + Number(hasParallel) !== 1) {
				return {
					content: [{ type: "text", text: "Provide exactly one mode: envelopePath for single dispatch, or dispatches[] for parallel dispatch." }],
					details: { agents: agents.map((agent) => ({ name: agent.name, description: agent.description })) },
				};
			}

			const requests: DispatchRequest[] = hasParallel
				? (params.dispatches as DispatchRequest[])
				: [
						{
							envelopePath: params.envelopePath!,
							agent: params.agent,
							model: params.model,
							tools: params.tools,
							cwd: params.cwd,
							includeContextFiles: params.includeContextFiles,
						},
					];

			if (requests.length > MAX_PARALLEL_DISPATCHES) {
				return {
					content: [{ type: "text", text: `Too many Foundry dispatches (${requests.length}); max is ${MAX_PARALLEL_DISPATCHES}.` }],
					details: { results: [] },
				};
			}

			const completed: DispatchResult[] = [];
			const emitProgress = () => {
				onUpdate?.({
					content: [{ type: "text", text: `Foundry team: ${completed.length}/${requests.length} dispatches complete` }],
					details: { results: [...completed] },
				});
			};

			const results = await mapWithConcurrency(requests, MAX_CONCURRENCY, async (request) => {
				const result = await runDispatch(ctx.cwd, agents, request, signal);
				completed.push(result);
				emitProgress();
				return result;
			});

			const failures = results.filter((result) => result.exitCode !== 0 || result.stopReason === "error" || result.stopReason === "aborted");
			const summary = results
				.map((result) => {
					const status = failures.includes(result) ? "failed" : "completed";
					const model = result.actualModel ? ` actual_model=${result.actualModel}` : "";
					return `### ${result.recipient} (${result.phase}) ${status}${model}\n\n${result.output}`;
				})
				.join("\n\n---\n\n");

			return {
				content: [{ type: "text", text: `Foundry team: ${results.length - failures.length}/${results.length} succeeded\n\n${summary}` }],
				details: { results },
				isError: failures.length > 0,
			};
		},
	});

	pi.registerCommand("foundry-agents", {
		description: "List Foundry package agents available to foundry_team",
		handler: async (_args, ctx) => {
			const agents = discoverFoundryAgents();
			ctx.ui.notify(`Foundry agents: ${agents.map((agent) => agent.name).join(", ") || "none"}`, "info");
		},
	});
}
