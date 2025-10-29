class Shortcut {
	static default_obj = {type:"Shortcut", styles:{cols:4, backgroundColor:"#2d2d38"}, content:[]};

	static init() {
		var style = document.createElement("style");
		document.head.appendChild(style);

		style.innerHTML = `
		.shortcut_container {
			display: grid;
			background-color: #2d2d38;
			width: calc(100% - 40px);
			border-radius: 20px;
			--cols: 4;
			--gap: 30px;
			--padding: 20px;
			--grid-column-width: calc((100% / var(--cols)) - ((var(--gap) * (var(--cols) - 1) ) / var(--cols)));
			grid-template-columns: repeat(var(--cols), var(--grid-column-width) );
			grid-gap: var(--gap);
			padding: var(--padding);
			max-width: calc(var(--cols) * 100px);
			margin: 0 auto;
		}

		.shortcut_link {
			display: flex;
			flex-direction: column;
			text-decoration: none;
			width: 100%;
		}

		.shortcut_link img {
			width: 100%;
			aspect-ratio: 1/1;
			object-fit: contain;
		}

		.shortcut_link p {
			color: white;
			text-align: center;
			margin: 0;
			margin-top: 5px;
			overflow: hidden;
		}
		`;
	}

	static load(element) {
		var container = this.createContainer(element);

		for (var i = 0; i < element.content.length; i++) {
			var link_data = element.content[i];
			var link = document.createElement("a");
			link.classList.add("shortcut_link");
			link.href = link_data.link;
			link.innerHTML = `<img src="${link_data.logo}"><p>${link_data.name}</p>`;

			container.appendChild(link);
		}

		return container;
	}

	static createContainer(element) {
		if (element == undefined) element = this.default_obj;

		var container = document.createElement("div");
		container.setAttribute("data-class", "Shortcut");
		container.classList.add("shortcut_container", "startpage_container");
		container.style.setProperty("--cols", element.styles.cols);
		container.style.setProperty("background-color", element.styles.backgroundColor);

		return container;
	}

	static startEdit(element) {
		element.classList.add("drag_container");
		element.addEventListener("dragover", dragOver);
		element.addEventListener("touchmove", dragOver);

		for (var i = 0; i < element.children.length; i++) {
			var link = element.children[i];
			link.classList.add("draggable_element");
			link.addEventListener("dragstart", dragStart);
			link.addEventListener("dragend", dragEnd);
			link.addEventListener("touchstart", dragStart);
			link.addEventListener("touchend", dragEnd);
		}
	}

	static stopEdit(element) {
		element.classList.remove("drag_container");
		element.removeEventListener("dragover", dragOver);
		element.removeEventListener("touchmove", dragOver);

		for (var i = 0; i < element.children.length; i++) {
			var link = element.children[i];
			link.classList.remove("draggable_element");
			link.removeEventListener("dragstart", dragStart);
			link.removeEventListener("dragend", dragEnd);
			link.removeEventListener("touchstart", dragStart);
			link.removeEventListener("touchend", dragEnd);
		}
	}
}