import { css, type ComponentChild, type FC } from "dreamland/core"
import { M3DlSection } from "./blitz-m3";

function A(this: FC<{ href: string, children: ComponentChild }>) {
	return (
		<a on:click={() => window.open(this.href)}>
			{this.children}
		</a>
	)
}

function Title(this: FC) {
	return (
		<div>
			<img src="https://flavorless.hackclub.com/logo.png" />
			<div class="title">Flavorless</div>
			<div class="desc">
				<p>
					A <A href="https://hackclub.com/">Hack Club</A> YSWS where you ship a <i>(functional)</i> website without any CSS, and Hack Club ships unknown prizes.
					More specifically, no inline styles or stylesheets are allowed, and no {'<canvas>'} except for OffscreenCanvas. SVG is allowed but you can't use any
					attributes related to CSS like fill/stroke. Javascript is allowed and encouraged.
				</p>
				<p>
					This website is a submission to Flavorless, mostly featuring the WASM HTML/CSS engine used to bypass the challenge rules.
					The official site is at <A href="https://flavorless.hackclub.com">flavorless.hackclub.com</A>.
				</p>
			</div>
		</div>
	)
}
Title.style = css`
	:scope {
		padding: 1rem 3rem;
		display: grid;
		grid-template-rows: min-content min-content;
		grid-template-columns: min-content 1fr;
		grid-template-areas:
			"img title"
			"img desc";
		column-gap: 1rem;
		background: #eee;
	}

	img {
		grid-area: img;
	}

	.title {
		font-size: 3.5rem;
		font-family: Jua;
		grid-area: title;
	}
	.desc {
		grid-area: desc;
		line-height: 1.4em;
	}

	@media (max-width: 50rem) {
		:scope {
			grid-template-areas: 
				"img title"
				"desc desc";
		}

		img {
			width: auto;
			height: 100%;
		}
	}
`;

export function BlitzApp(this: FC<{}, { count: number, text: string }>) {
	this.count = 0;
	this.text = "abc";
	use(this.text).constrain(this).listen(console.log)
	return (
		<div>
			<Title />
			<div class="section">
				<div class="body">
					<div class="title">About this site</div>
					<p>
						This site uses the <A href="https://blitz.is">Blitz HTML/CSS engine</A> compiled to WebAssembly to render regular dreamland.js pages through
						a custom DOM proxy that forwards to Blitz's DOM. Blitz renders to an OffscreenCanvas (which is allowed per the rules), but it doesn't go straight
						to a {'<canvas>'} since that's not allowed. Instead the browser renders a {'<video>'} element, constantly pulling video frames from a custom
						MediaStreamTrackGenerator stream that exports the OffscreenCanvas as an ImageBitmap. This allows for (mostly?) zero-copy rendering.
						Events are forwarded through the usual JavaScript event handlers.
					</p>
					<p>
						Blitz supports a lot of the common HTML elements and CSS rules, so it's possible to just reuse existing code built for the browser.
						I've embedded a m3-dreamland Card with a calculator in it using the same code that I would outside of Blitz, and it functions normally.
					</p>
					<p>
						You can also visit (a slightly modified version of) the Flavortown homepage: <a on:click={() => location.search = "?flavortown"}>Check it out</a>!
						It's pretty slow at rendering since the page is so big though.
						Try <a on:click={() => (self as any).deleteCache()}>clearing the blitz-net HTTP cache</a> if it doesn't work.
					</p>
				</div>
				<M3DlSection />
			</div>
		</div>
	)
}
BlitzApp.style = css`
	:scope {
		background: #ddd;
		height: 100%;
		overflow: scroll;
	}

	.section {
		display: flex;
		padding: 2rem;
		gap: 1rem;
		align-items: center;
	}

	.body {
		align-self: start;
		flex: 1;
		line-height: 1.4em;
	}

	.title {
		font-size: 2rem;
		font-family: Jua;
	}

	@media (max-width: 50rem) {
		.section {
			flex-direction: column;
		}
	}
`;
