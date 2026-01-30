import type { FC } from "dreamland/core";

export interface ImageStream {
	(): Promise<{ value?: ImageBitmap, done: boolean }>;
	scale: number,
}

function rafPipe(stream: ImageStream): ReadableStream<VideoFrame> {
	let scale = stream.scale;
	return new ReadableStream({
		async start(controller) {
			while (true) {
				let rafTime = await new Promise<number>(r => requestAnimationFrame(r));
				let { value, done } = await stream();
				if (done) break;
				if (!value) continue;

				controller.enqueue(new VideoFrame(value, { timestamp: rafTime, displayWidth: value.width / scale, displayHeight: value.height / scale }));
				value.close();
			}
			controller.close();
		},
	})
}
function streamToTrack(stream: ReadableStream<VideoFrame>): MediaStreamVideoTrack {
	let generator = new MediaStreamTrackGenerator({ kind: "video" });
	stream.pipeTo(generator.writable).catch(err => console.error("Stream error:", err));
	return generator;
}

export function FakeCanvas(this: FC<{
	stream: ImageStream,
	pointer: (e: PointerEvent, x: number, y: number) => void,
	scroll: (e: WheelEvent, x: number, y: number) => void,
	key: (e: KeyboardEvent) => void,
}>) {
	let stream = new MediaStream();
	stream.addTrack(streamToTrack(rafPipe(this.stream)));

	let pointer = (e: PointerEvent) => {
		let { x, y } = this.root.getBoundingClientRect();
		this.pointer(e, x, y);
	};

	let wheel = (e: WheelEvent) => {
		let { x, y } = this.root.getBoundingClientRect();
		this.scroll(e, x, y);
	};

	return (
		<video
			attr:srcObject={stream}

			autoplay="true"
			playsinline="true"
			attr:muted={true}
			on:pause={(e: any) => e.target.play()}

			on:pointerdown={pointer}
			on:pointerup={pointer}
			on:pointermove={pointer}
			on:keydown={this.key}
			on:keyup={this.key}
			on:wheel={wheel}
		/>
	)
}
