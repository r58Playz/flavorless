import type { FC } from "dreamland/core";

function rafPipe<T>(stream: ReadableStream<T>): ReadableStream<T> {
	let reader = stream.getReader();
	return new ReadableStream({
		async start(controller) {
			while (true) {
				await new Promise(r => requestAnimationFrame(r));
				let { value, done } = await reader.read();
				if (!value || done) break;
				controller.enqueue(value);
			}
			controller.close();
		},
		cancel() {
			reader.cancel();
		}
	}, new CountQueuingStrategy({ highWaterMark: 1 }))
}
function streamToTrack(stream: ReadableStream<ImageBitmap>): MediaStreamVideoTrack {
	let i = 0;
	let generator = new MediaStreamTrackGenerator({ kind: "video" });
	stream.pipeThrough(new TransformStream({
		transform(chunk, controller) {
			const frame = new VideoFrame(chunk, { timestamp: i++ });
			controller.enqueue(frame);
			chunk.close();
		}
	})).pipeTo(generator.writable).catch(err => console.error("Stream error:", err));
	return generator;
}

export function FakeCanvas(this: FC<{ stream: ReadableStream<ImageBitmap> }>) {
	let stream = new MediaStream();
	stream.addTrack(streamToTrack(rafPipe(this.stream)));

	return (
		<video 
			attr:srcObject={stream}

			autoplay="true"
			playsinline="true"
			attr:muted={true}
			on:pause={(e: any) => e.target.play()}
		/>
	)
}
