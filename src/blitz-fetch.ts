import epoxyInit, { EpoxyClient, EpoxyClientOptions } from "@mercuryworkshop/epoxy-tls/minimal-epoxy";
// @ts-ignore
import epoxyWasm from "../node_modules/@mercuryworkshop/epoxy-tls/minimal/epoxy.wasm?url";

let initted = false;
let client: EpoxyClient | undefined;
let clientPromiseResolve = () => {};
let clientPromise = new Promise<void>(r => clientPromiseResolve = r);
let inflight = 0;

let cache = await caches.open("blitz-net");

(self as any).deleteCache = () => { caches.delete("blitz-net"); location.reload(); }

export async function initBlitzNet(wisp: string) {
	console.log("initting blitz net with", wisp);
	if (!initted) {
		await epoxyInit({ module_or_path: epoxyWasm });
		initted = true;
	}

	let opts = new EpoxyClientOptions();
	// ttf fonts
	opts.user_agent = "curl/8.18.0";
	let epx = new EpoxyClient(wisp, opts);
	try {
		if (!(await epx.fetch("https://example.com/", {}).then(r => r.text())).startsWith(`<!doctype html><html lang="en"><head><title>Example Domain</title>`)) {
			throw "";
		}
	} catch {
		throw "invalid wisp";
	}

	client = epx;
	clientPromiseResolve();
}

export async function blitzFetch(req: Request): Promise<[string, Uint8Array]> {
	await clientPromise;
	if (!client) throw "client not initted";
	inflight++;
	try {
		console.debug("[blitz-net]", req.method, req.url)

		let res;
		if (!(res = await cache.match(req))) {
			res = await client.fetch(req.url, { method: req.method, headers: req.headers, body: req.body });
			cache.put(req, res.clone());
		}

		return [res.url || req.url, new Uint8Array(await res.arrayBuffer())];
	} finally {
		inflight--;
	}
}

export function blitzInflight(): boolean {
	return inflight > 0;
}
