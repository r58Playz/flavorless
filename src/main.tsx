import type { FC } from "dreamland/core";
import { FakeCanvas, type ImageStream } from "./fakecanvas";

import init, { BlitzRenderer } from "../blitz/pkg/blitz_dl";
// @ts-ignore
import blitz_wasm from "../blitz/pkg/blitz_dl.wasm?url";
import initialHtml from "./initial.html?raw";

await init({ module_or_path: blitz_wasm });

let SCALE = Math.ceil(window.devicePixelRatio);

function SvgState(this: FC<{ state: string }>) {
	return (
		<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%">
			<text
				x="50%"
				y="50%"
				text-anchor="middle"
				dominant-baseline="middle"
				font-size="3vw"
				fillcolor="#333"
				font-family="monospace"
				font-weight="bold"
			>
				renderer state: {this.state}
			</text>
		</svg>
	)
}

function App(this: FC<{}, { state: "initting" | "rendering" | "resize-debounce", dims: [number, number] }>) {
	this.state = "initting" as any;

	let pointer: [PointerEvent, number, number][] = [];
	let wheel: [WheelEvent, number, number][] = [];
	let key: [KeyboardEvent][] = [];

	let screen: OffscreenCanvas | undefined;
	let renderer: BlitzRenderer | undefined;
	let stream: ImageStream = (async () => {
		if (!screen || !renderer) return { done: false };
		if (this.state === "initting") this.state = "rendering";

		for (let ev of pointer.splice(0)) renderer.event(renderer.event_pointer(...ev))
		for (let ev of wheel.splice(0)) renderer.event(renderer.event_wheel(...ev))
		for (let ev of key.splice(0)) renderer.event(renderer.event_keyboard(...ev))

		renderer.render(performance.now());

		return { value: screen.transferToImageBitmap(), done: false };
	}) as any;
	stream.scale = SCALE;

	this.cx.mount = async () => {
		let html: HTMLHtmlElement = document.childNodes[0] as any;
		this.dims = [html.clientWidth, html.clientHeight];

		let debounce: number | undefined;
		window.addEventListener("resize", () => {
			this.state = "resize-debounce";
			if (debounce) clearTimeout(debounce);

			debounce = setTimeout(() => {
				debounce = undefined;
				this.dims = [html.clientWidth, html.clientHeight];
			}, 1000)
		})
	}

	use(this.dims).constrain(this).listen(async ([width, height]) => {
		this.state = "initting";
		console.log("creating pipeline with dims", width, height);
		let lastRenderer = renderer;
		renderer = undefined;

		screen = new OffscreenCanvas(width * SCALE, height * SCALE);
		if (!lastRenderer) {
			lastRenderer = await BlitzRenderer.new(initialHtml, screen, SCALE);
		} else {
			await lastRenderer.resize(screen, SCALE);
		}

		renderer = lastRenderer;
	})

	let onEv = <T extends Array<any>, >(arr: T[]) => (...args: T) => arr.push(args);

	return (
		<div>
			{use(this.state).map(x => x !== "rendering").and(x => <SvgState state={this.state} />)}
			{use(this.state).map(x => x === "rendering").and(<FakeCanvas stream={stream} pointer={onEv(pointer)} scroll={onEv(wheel)} key={onEv(key)} />)}
		</div>
	)
}
document.body.replaceWith(<App />);
