import { css, type FC } from "dreamland/core"

export function BlitzApp(this: FC<{}, { count: number }>) {
	this.count = 0;
	return (
		<div>
			<div class="title">dreamland.js in Blitz loaded!</div>
			<p>
				Counter app!
			</p>
			<button on:click={() => this.count++}>Count: {use(this.count)}</button>
		</div>
	)
}
BlitzApp.style = css`
	.title {
		font-size: 2rem;
	}
`;
