import epoxyInit, { EpoxyClient, EpoxyClientOptions } from "@mercuryworkshop/epoxy-tls/minimal-epoxy";
// @ts-ignore
import epoxyWasm from "../node_modules/@mercuryworkshop/epoxy-tls/minimal/epoxy.wasm?url";

let initted = false;
let client: EpoxyClient | undefined;

export async function initBlitzNet(wisp: string) {
	console.log("initting blitz net with", wisp);
	if (!initted) {
		await epoxyInit({ module_or_path: epoxyWasm });
		initted = true;
	}

	let opts = new EpoxyClientOptions();
	let epx = new EpoxyClient(wisp, opts);
	try {
		if (!(await epx.fetch("https://example.com/", {}).then(r => r.text())).startsWith(`<!doctype html><html lang="en"><head><title>Example Domain</title>`)) {
			throw "";
		}
	} catch {
		throw "invalid wisp";
	}

	client = epx;
}

export async function blitzFetch(req: Request): Promise<[string, Uint8Array]> {
	if (!client) throw "client not initted";
}
