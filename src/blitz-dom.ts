import type { DomImpl } from "dreamland/core";
import { getDomImpl } from "dreamland/core";
import { BlitzDocument, BlitzNode } from "../blitz/pkg/blitz_dl";

// @ts-expect-error rrweb-cssom doesn't have types
import { CSSOM } from "virtual:rrweb-cssom";

// hacky thing to let the state changes through
export let withHarnessDisabled = (func: () => void) => {
	disableHarness = true;
	func();
	disableHarness = false;
};
let disableHarness = false;

export class BlitzDomNode {
	node: BlitzNode;
	doc: BlitzDocument;
	sheet?: any;

	constructor(node: BlitzNode, doc: BlitzDocument) {
		this.node = node;
		this.doc = doc;
	}

	get parentNode(): BlitzDomNode | undefined {
		let parent = this.node.parent(this.doc);
		return parent ? new BlitzDomNode(parent, this.doc) : undefined;
	}

	get firstChild(): BlitzDomNode | undefined {
		let children = this.node.children(this.doc);
		return children.length > 0 ? new BlitzDomNode(children[0], this.doc) : undefined;
	}

	get nextSibling(): BlitzDomNode | undefined {
		let next = this.node.next_sibling(this.doc);
		return next ? new BlitzDomNode(next, this.doc) : undefined;
	}

	get childNodes() {
		return this.node.children(this.doc).map(n => new BlitzDomNode(n, this.doc));
	}

	appendChild(child: BlitzDomNode) {
		this.node.append(this.doc, child.node);
		return child;
	}

	append(child: BlitzDomNode) {
		return this.appendChild(child);
	}

	removeChild(child: BlitzDomNode) {
		this.node.remove(this.doc, child.node);
	}

	insertBefore(child: BlitzDomNode, anchor: BlitzDomNode) {
		this.node.insert(this.doc, child.node, anchor.node);
	}

	replaceWith(el: BlitzDomNode) {
		this.node.replace(this.doc, el.node);
	}

	setAttribute(key: string, value: string) {
		this.node.set_attribute(this.doc, key, value);
	}

	removeAttribute(key: string) {
		this.node.remove_attribute(this.doc, key);
	}

	getAttribute(key: string): string | undefined {
		return this.node.get_attribute(this.doc, key) ?? undefined;
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
		this.node.add_event_listener(this.doc, event, handler);
	}

	removeEventListener(event: string, handler: Function) {
		this.node.remove_event_listener(this.doc, event, handler);
	}

	get outerHTML() {
		return this.node.get_outer_html(this.doc);
	}

	get innerHTML() {
		return this.node.get_inner_html(this.doc);
	}

	set innerHTML(value: string) {
		this.node.set_inner_html(this.doc, value);
	}

	set innerText(value: string) {
		if (this.sheet) {
			this.sheet = CSSOM.parse(value);
			this.node.set_inner_text(this.doc, this.sheet.toString());
		} else {
			this.node.set_inner_text(this.doc, value);
		}
	}

	get data() {
		return this.node.get_data(this.doc) ?? "";
	}

	set data(value: string) {
		this.node.set_data(this.doc, value);
	}
}

export function createBlitzDomImpl(doc: BlitzDocument): DomImpl {
	let oldImpl = getDomImpl();

	return [
		{
			createElement(type: string) {
				if (disableHarness) return document.createElement(type);

				let node = BlitzNode.new(doc, type);
				let wrapper = new BlitzDomNode(node, doc);
				
				if (type === "style") {
					wrapper.sheet = new CSSOM.CSSStyleSheet();
				}
				
				return wrapper;
			},
			createElementNS(ns: string, type: string) {
				if (disableHarness) return document.createElementNS(ns, type);

				let node = BlitzNode.new_ns(doc, type, ns);
				return new BlitzDomNode(node, doc);
			},
			head: new BlitzDomNode(doc.query_selector("head")!, doc),
		},
		BlitzDomNode,
		(text?: any) => {
			if (disableHarness && text instanceof Node) return text;
			if (disableHarness) return new Text(text);

			let node = BlitzNode.new_text(doc, String(text ?? ""));

			return new BlitzDomNode(node, doc);
		},
		(text?: any) => {
			if (disableHarness) return new Comment(text);

			let node = BlitzNode.new_comment(doc);
			let wrapper = new BlitzDomNode(node, doc);
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
