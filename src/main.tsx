import type { FC } from "dreamland/core";
import { FakeCanvas } from "./fakecanvas";

import init, { BlitzRenderer } from "../blitz/pkg/blitz_dl";
// @ts-ignore
import blitz_wasm from "../blitz/pkg/blitz_dl.wasm?url";

await init({ module_or_path: blitz_wasm });

function App(this: FC) {
	let screen: OffscreenCanvas;
	let renderer: BlitzRenderer;
	let stream = new ReadableStream({
		async start() {
			screen = new OffscreenCanvas(window.innerWidth, window.innerHeight);
			renderer = await BlitzRenderer.with_offscreen(screen);
		},
		pull(controller) {
			if (!screen || !renderer) return;

			renderer.render();

			let map = screen.transferToImageBitmap();
			controller.enqueue(map);
		}
	})

	return (
		<div>
			<FakeCanvas stream={stream} />
		</div>
	)
}
document.body.replaceWith(<App />);
