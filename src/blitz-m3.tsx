import { Button, ButtonGroup, Card, DynamicScheme, Hct, Icon, SchemeStyles, Variant } from "m3-dreamland";
import { css, type FC } from "dreamland/core"
import iconDelete from "@ktibow/iconset-material-symbols/delete";

let scheme = new DynamicScheme({
	sourceColorHct: Hct.fromInt(0x00c875a1),
	contrastLevel: 0,
	specVersion: "2025",
	variant: Variant.VIBRANT,
	isDark: true,
});

export function M3DlSection(this: FC<{}, { field: number, stack: number, op: "+" | "-" | "*" | "/" | undefined }>) {
	let PRECISION = 1000000000;

	let num = (digit: number) => () => {
		// ???
		if (disabled.value) return;
		this.field *= 10;
		this.field += digit;
	};
	let ac = () => {
		this.field = 0;
		this.stack = 0;
		this.op = undefined;
	}
	let op = (op: typeof this.op) => () => {
		if (this.stack) execute()
		this.stack = this.field;
		this.op = op;
		this.field = 0;
	};
	let execute = () => {
		if (!this.op) return;
		try {
			let a = this.stack;
			let b = this.field;
			let op = this.op;
			ac();
			if (op === "+")
				this.field = a + b;
			else if (op === "-")
				this.field = a - b;
			else if (op === "*")
				this.field = a * b;
			else
				this.field = a / b;
		} catch {}
	};

	ac();

	let disabled = use(this.field).map(x => x > PRECISION * 10);

	return (
		<div>
			<SchemeStyles scheme={scheme} motion="expressive">
				<Card variant="elevated">
					<div class="m3dl-font-headline-large m3dl-header">M3 Expressive Calculator</div>
					<div class="m3dl-font-display-large result">
						<Button variant="outlined" size="m" icon="full" on:click={() => this.field = 0}><Icon icon={iconDelete} /></Button>
						<span>
							{use(this.field).map(x => String(Math.round(x * PRECISION) / PRECISION))}
						</span>
					</div>
					<div class="buttons">
						<ButtonGroup variant="standard" size="l">
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(1)}>1</Button>
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(2)}>2</Button>
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(3)}>3</Button>
							<Button variant="tonal" size="l" icon="full" on:click={op("+")}>+</Button>
						</ButtonGroup>
						<ButtonGroup variant="standard" size="l">
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(4)}>4</Button>
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(5)}>5</Button>
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(6)}>6</Button>
							<Button variant="tonal" size="l" icon="full" on:click={op("-")}>-</Button>
						</ButtonGroup>
						<ButtonGroup variant="standard" size="l">
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(7)}>7</Button>
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(8)}>8</Button>
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(9)}>9</Button>
							<Button variant="tonal" size="l" icon="full" on:click={op("*")}>x</Button>
						</ButtonGroup>
						<ButtonGroup variant="standard" size="l">
							<Button disabled={disabled} variant="outlined" size="l" icon="full" on:click={num(0)}>0</Button>
							<Button variant="filled" size="l" icon="full" on:click={ac}>AC</Button>
							<Button variant="filled" size="l" icon="full" on:click={execute}>=</Button>
							<Button variant="tonal" size="l" icon="full" on:click={op("/")}>/</Button>
						</ButtonGroup>
					</div>
				</Card>
			</SchemeStyles>
		</div>
	)
}
M3DlSection.style = css`
	:scope {
		width: 30rem;
	}

	:scope > :global(.m3dl-scheme-styles) {
		font-family: var(--m3dl-font);

		color: rgb(var(--m3dl-color-on-background));
		
		padding: 1em;
	}

	:global(.m3dl-buttongroup) {
		--m3dl-button-multiplier: 0.75 !important;
	}

	.buttons {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.result {
		display: flex;
		align-items: center;
		justify-content: space-between;

		padding: 1rem 0;
	}
`;
