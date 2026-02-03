import type { DomImpl } from "dreamland/core";
import { getDomImpl } from "dreamland/core";
import { BlitzDocument, BlitzEventHandler, BlitzNode } from "../blitz/pkg/blitz_dl";

// @ts-expect-error rrweb-cssom doesn't have types
import { CSSOM } from "virtual:rrweb-cssom";

// hacky thing to let the state changes through
export let withHarnessDisabled = (func: () => void) => {
	disableHarness = true;
	func();
	disableHarness = false;
};
let disableHarness = false;

let DOC: BlitzDocument;
let EVENTS: BlitzEventHandler;

export class BlitzDomNode {
	node: BlitzNode;
	sheet?: any;

	constructor(node: BlitzNode) {
		this.node = node;
	}

	get parentNode(): BlitzDomNode | undefined {
		let parent = this.node.parent(DOC);
		return parent ? new BlitzDomNode(parent) : undefined;
	}

	get firstChild(): BlitzDomNode | undefined {
		let children = this.node.children(DOC);
		return children.length > 0 ? new BlitzDomNode(children[0]) : undefined;
	}

	get nextSibling(): BlitzDomNode | undefined {
		let next = this.node.next_sibling(DOC);
		return next ? new BlitzDomNode(next) : undefined;
	}

	get childNodes() {
		return this.node.children(DOC).map(n => new BlitzDomNode(n));
	}

	appendChild(child: BlitzDomNode) {
		this.node.append(DOC, child.node);
		return child;
	}

	append(child: BlitzDomNode) {
		return this.appendChild(child);
	}

	removeChild(child: BlitzDomNode) {
		this.node.remove(DOC, child.node);
	}

	insertBefore(child: BlitzDomNode, anchor: BlitzDomNode) {
		this.node.insert(DOC, child.node, anchor.node);
	}

	replaceWith(el: BlitzDomNode) {
		this.node.replace(DOC, el.node);
	}

	setAttribute(key: string, value: string) {
		this.node.set_attribute(DOC, key, value);
	}

	removeAttribute(key: string) {
		this.node.remove_attribute(DOC, key);
	}

	getAttribute(key: string): string | undefined {
		return this.node.get_attribute(DOC, key) ?? undefined;
	}

	get classList() {
		const getClasses = () => {
			const classAttr = this.getAttribute("class") || "";
			return classAttr.split(/\s+/).filter(c => c);
		};
		
		return {
			add: (...tokens: string[]) => {
				const current = new Set(getClasses());
				tokens.forEach(t => current.add(t));
				this.setAttribute("class", Array.from(current).join(" "));
			},
			remove: (...tokens: string[]) => {
				const current = new Set(getClasses());
				tokens.forEach(t => current.delete(t));
				this.setAttribute("class", Array.from(current).join(" "));
			},
			toggle: (token: string, force?: boolean) => {
				const current = new Set(getClasses());
				if (force === true || (force === undefined && !current.has(token))) {
					current.add(token);
					this.setAttribute("class", Array.from(current).join(" "));
					return true;
				} else {
					current.delete(token);
					this.setAttribute("class", Array.from(current).join(" "));
					return false;
				}
			},
			contains: (token: string) => getClasses().includes(token),
			replace: (oldToken: string, newToken: string) => {
				const current = new Set(getClasses());
				if (current.has(oldToken)) {
					current.delete(oldToken);
					current.add(newToken);
					this.setAttribute("class", Array.from(current).join(" "));
					return true;
				}
				return false;
			},
			get length() { return getClasses().length; },
			item: (index: number) => getClasses()[index] ?? null,
			toString: () => this.getAttribute("class") || "",
			[Symbol.iterator]: function*() {
				yield* getClasses();
			}
		};
	}

	addEventListener(event: string, handler: Function) {
		this.node.add_event_listener(EVENTS, event, handler);
	}

	removeEventListener(event: string, handler: Function) {
		this.node.remove_event_listener(EVENTS, event, handler);
	}

	get outerHTML() {
		return this.node.get_outer_html(DOC);
	}

	get innerHTML() {
		return this.node.get_inner_html(DOC);
	}

	set innerHTML(value: string) {
		this.node.set_inner_html(DOC, value);
	}

	set innerText(value: string) {
		if (this.sheet) {
			this.sheet = CSSOM.parse(value);
			this.node.set_inner_text(DOC, this.sheet.toString());
		} else {
			this.node.set_inner_text(DOC, value);
		}
	}

	get data() {
		return this.node.get_data(DOC) ?? "";
	}

	set data(value: string) {
		this.node.set_data(DOC, value);
	}
}

export function createBlitzDomImpl(doc: BlitzDocument, events: BlitzEventHandler): DomImpl {
	let oldImpl = getDomImpl();

	DOC = doc;
	EVENTS = events;

	events.set_doc_overrider((newDoc: any) => {
		let old = DOC;
		DOC = newDoc;
		return old;
	})

	return [
		{
			createElement(type: string) {
				if (disableHarness) return document.createElement(type);

				let node = BlitzNode.new(DOC, type);
				let wrapper = new BlitzDomNode(node);
				
				if (type === "style") {
					wrapper.sheet = new CSSOM.CSSStyleSheet();
				}
				
				return wrapper;
			},
			createElementNS(ns: string, type: string) {
				if (disableHarness) return document.createElementNS(ns, type);

				let node = BlitzNode.new_ns(DOC, type, ns);
				return new BlitzDomNode(node);
			},
			head: new BlitzDomNode(DOC.query_selector("head")!),
		},
		BlitzDomNode,
		(text?: any) => {
			if (disableHarness && text instanceof Node) return text;
			if (disableHarness) return new Text(text);

			let node = BlitzNode.new_text(DOC, String(text ?? ""));

			return new BlitzDomNode(node);
		},
		(text?: any) => {
			if (disableHarness) return new Comment(text);

			let node = BlitzNode.new_comment(DOC);
			let wrapper = new BlitzDomNode(node);
			if (text != null) {
				wrapper.data = String(text);
			}
			return wrapper;
		},
		oldImpl[4],
		undefined,
		undefined,
	] as const satisfies DomImpl;
}
