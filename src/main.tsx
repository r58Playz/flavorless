import type { FC } from "dreamland/core";
import { FakeCanvas } from "./fakecanvas";

function App(this: FC) {
	let canvas = new OffscreenCanvas(512, 512);
	let ctx = canvas.getContext("2d")!;

	let last = performance.now();
	let t = 0;
	let stream = new ReadableStream({
		pull(controller) {
			let now = performance.now();
			let frametime = now - last;
			last = now;

			t += frametime * 0.002;
			let r = Math.floor((Math.sin(t) * 0.5 + 0.5) * 255);
			let g = Math.floor((Math.sin(t + 2) * 0.5 + 0.5) * 255);
			let b = Math.floor((Math.sin(t + 4) * 0.5 + 0.5) * 255);

			ctx.clearRect(0, 0, 512, 512);

			// Draw centered rectangle
			ctx.fillStyle = `rgb(${r},${g},${b})`;
			ctx.fillRect(128, 128, 256, 256);

			// Frame time text
			ctx.fillStyle = "black";
			ctx.font = `${(Math.abs(Math.sin(t)) * 5) + 15}px monospace`;
			ctx.fillText(`Frame time: ${frametime.toFixed(2)} ms`, 10, 30);

			let map = canvas.transferToImageBitmap();
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
