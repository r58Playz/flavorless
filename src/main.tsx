import { setDomImpl, type FC } from "dreamland/core";
import { FakeCanvas, type ImageStream } from "./fakecanvas";

import init, { BlitzDocument, BlitzRenderer, type BlitzRendererResult } from "../blitz/pkg/blitz_dl";
// @ts-ignore
import blitz_wasm from "../blitz/pkg/blitz_dl.wasm?url";
import initialHtml from "./initial.html?raw";
import { BlitzDomNode, createBlitzDomImpl, withHarnessDisabled } from "./blitz-dom";
import { BlitzApp } from "./blitz-main";

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

function App(this: FC<{ ret: BlitzRendererResult, ready: () => void, }, { state: "initting" | "rendering" | "resize-debounce", dims: [number, number] }>) {
	withHarnessDisabled(() => this.state = "initting");
	let ready = false;

	let pointer: [PointerEvent, number, number][] = [];
	let wheel: [WheelEvent, number, number][] = [];
	let key: [KeyboardEvent][] = [];

	let screen: OffscreenCanvas | undefined;
	let [renderer, doc, events] = this.ret;
	let stream: ImageStream = (async () => {
		if (!screen) return { done: false };
		if (this.state === "initting") withHarnessDisabled(() => this.state = "rendering");

		for (let ev of pointer.splice(0)) doc.event(events, BlitzDocument.event_pointer(...ev))
		for (let ev of wheel.splice(0)) doc.event(events, BlitzDocument.event_wheel(...ev))
		for (let ev of key.splice(0)) doc.event(events, BlitzDocument.event_keyboard(...ev))

		renderer.render(doc, performance.now());

		return { value: screen.transferToImageBitmap(), done: false };
	}) as any;
	stream.scale = SCALE;

	this.cx.mount = async () => {
		let html: HTMLHtmlElement = document.childNodes[0] as any;
		this.dims = [html.clientWidth, html.clientHeight];

		let debounce: number | undefined;
		window.addEventListener("resize", () => {
			withHarnessDisabled(() => this.state = "resize-debounce");
			if (debounce) clearTimeout(debounce);

			debounce = setTimeout(() => {
				debounce = undefined;
				this.dims = [html.clientWidth, html.clientHeight];
			}, 1000)
		})
	}

	use(this.dims).constrain(this).listen(async ([width, height]) => {
		withHarnessDisabled(() => this.state = "initting");
		screen = undefined;

		console.log("creating pipeline with dims", width, height);
		let currentScreen = new OffscreenCanvas(width * SCALE, height * SCALE);
		await renderer.resize(doc, currentScreen, SCALE);

		screen = currentScreen;

		if (!ready) {
			ready = true;
			this.ready();
		}
	})

	let onEv = <T extends Array<any>,>(arr: T[]) => (...args: T) => arr.push(args);

	return (
		<div>
			{use(this.state).map(x => x !== "rendering").and(_ => <SvgState state={this.state} />)}
			{use(this.state).map(x => x === "rendering").and(<FakeCanvas stream={stream} pointer={onEv(pointer)} scroll={onEv(wheel)} key={onEv(key)} />)}
		</div>
	)
}

let renderer = await BlitzRenderer.new(initialHtml, new OffscreenCanvas(1, 1), 1);
document.body.replaceWith(<App ret={renderer} ready={() => {
	console.log("ready...");
	let dom = renderer[1];
	let events = renderer[2];
	let impl = createBlitzDomImpl(dom, events);

	setTimeout(() => {
		setDomImpl(impl);
		let app = <BlitzApp /> as any as BlitzDomNode;
		console.log(app.outerHTML);
		dom.query_selector("#app")!.replace(dom, app.node);
	}, 1000);
}} />);
