// All custom ids, classes, css rules must be started with the class name
//
// Needed methods
// - init					-> runs on page load to add styles etc.
// - load					-> add elements according to user data. Must return container element
// - startEdit				-> start edit of provided element
// - stopEdit				-> finish edit of provided element
// - createContainer		-> create new item element. Can show edit menu
// - loadContainer			-> create new container element. Must return container element
//
// Needed attributes
// - default_obj			-> default user data object
// - label_name				-> label for the button which creates a new item
//
// Provided functions/ variables
// - edit_mode_active		-> returns true when edit mode is active
// - editContainer			-> add edit attributes to provided container
// - editElement			-> add edit attributes to provided item
// - finishEditContainer	-> remove edit attributes to provided container
// - finishEditElement		-> remove edit attributes to provided item
// - showDialog				-> returns builtin dialog element and shows it, closable with dialog.close(), do not overwrite content
//

class Shortcut {
	static default_obj = {type:"Shortcut", styles:{width:"100px", cols:4, backgroundColor:"#2d2d38"}, content:[]};
	static label_name = "Add Shortcut";

	static init() {
		var style = document.createElement("style");
		document.head.appendChild(style);

		style.innerHTML = `
		.startpage_wrapper:has(.shortcut_container) {
			background-color: #2d2d38;
			border-radius: 20px;
			margin: 0 auto;
		}

		.shortcut_container {
			display: grid;
			--width: 100px;
			--cols: 4;
			--gap: 30px;
			--padding: 20px;
			--grid-column-width: calc((100% / var(--cols)) - ((var(--gap) * (var(--cols) - 1) ) / var(--cols)));
			grid-template-columns: repeat(var(--cols), var(--grid-column-width) );
			grid-gap: var(--gap);
			padding: var(--padding);
			width: calc(var(--cols) * var(--width));
		}

		.shortcut_link, .shortcut_create {
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

		.shortcut_label {
			display: block;
			margin: 10px auto;
		}

		.shortcut_create {
			display: none;
			height: 100%;
			cursor: pointer;
			/*background-color: var(--backgroundColor);*/
		}

		.edit_mode .shortcut_create {
			display: flex;
		}
		`;
	}

	static load(element) {
		var container = this.loadContainer(element);

		for (var i = 0; i < element.content.length; i++) {
			var link = this.loadElement(element.content[i]);
			container.appendChild(link);
		}

		return container;
	}

	static loadContainer(element) {
		if (element == undefined) element = this.default_obj;

		var container = document.createElement("div");
		container.classList.add("shortcut_container");
		container.style.setProperty("--cols", element.styles.cols);
		container.style.setProperty("--width", element.styles.width);

		// edit container elements
		var create_button = document.createElement("button");
		create_button.innerHTML = '<img src="img/add.svg">';
		create_button.style.setProperty("right", "10px");
		create_button.classList.add("edit_container_button");
		create_button.addEventListener("click", () => {
			Shortcut.createElement(container, undefined, false);
		});

		var edit_button = document.createElement("button");
		edit_button.innerHTML = '<img src="img/edit.svg">';
		create_button.style.setProperty("right", "40px");
		edit_button.classList.add("edit_container_button");
		edit_button.addEventListener("click", () => {
			Shortcut.editContainer(container);
		});

		var button_box = document.createElement("div");
		button_box.classList.add("button_box");
		button_box.append(create_button);
		button_box.append(edit_button);

		if (edit_mode_active) editContainer(container);

		var wrapper = document.createElement("div");
		wrapper.setAttribute("data-class", "Shortcut");
		wrapper.classList.add("startpage_wrapper");
		wrapper.style.setProperty("background-color", element.styles.backgroundColor);

		wrapper.append(button_box);
		wrapper.append(container);

		return container;
	}

	static loadElement(link_data) {
		var link = document.createElement("a");
		link.classList.add("shortcut_link");
		link.href = link_data.link;
		link.innerHTML = `<img src="${link_data.logo}"><p>${link_data.name}</p>`;

		if (edit_mode_active) this.editElement(link);
		return link;
	}

	static editContainer(container) {
		var dialog = showDialog("any");
		var json = window.localStorage.getItem(location.pathname);
		var data = JSON.parse(json);

		var width = container.style.getPropertyValue("--width");
		var cols = container.style.getPropertyValue("--cols");

		var html = `
			<p>Edit Group</p>
			<label class="shortcut_label">Shortcut width: <input type="text" id="shortcut_width" value="${width}"></label>
			<label class="shortcut_label">Columns amount: <input type="text" id="shortcut_cols" value="${cols}"></label>
		`;
		var div = document.createElement("div");
		div.innerHTML = html;

		var button = document.createElement("button");
		button.innerHTML = "Update";
		button.addEventListener("click", () => {
			// store input
			var {wrapper_index} = getPosition(container);
			var container_obj = data.elements[wrapper_index];

			container_obj.styles.width = document.getElementById("shortcut_width").value;
			container_obj.styles.cols = document.getElementById("shortcut_cols").value;

			window.localStorage.setItem(location.pathname, JSON.stringify(data));

			// show input
			container.style.setProperty("--width", container_obj.styles.width);
			container.style.setProperty("--cols", container_obj.styles.cols);

			dialog.close();
		});

		dialog.appendChild(div);
		dialog.appendChild(button);
	}

	static editElement(element) {
		editElement(element);
		element.addEventListener("click", Shortcut.createElementHelper);
	}

	static createElementHelper(e) {
		e.preventDefault();
		var element = e.currentTarget;
		Shortcut.createElement(element.parentElement, element, false);
	}

	static createElement(container, element, new_container) {
		if (new_container) var dialog = showDialog("needed");
		else var dialog = showDialog("any");

		var json = window.localStorage.getItem(location.pathname);
		var data = JSON.parse(json);

		var {wrapper_index, element_index} = getPosition(container, element);

		// determine if element has to be created or edited
		if (element == undefined) {
			var edit_element = false
			var element_obj = {name:"", link:"", logo:""};
			var html_title = "Add Shortcut";
			var html_button = "Create";
		}
		else {
			var edit_element = true;
			var element_obj = data.elements[wrapper_index].content[element_index];
			var html_title = "Edit Shortcut";
			var html_button = "Update";
		}
		
		// create html
		var html = `
			<p>${html_title}</p>
			<label class="shortcut_label">Name: <input type="text" id="shortcut_name" value="${element_obj.name}"></label>
			<label class="shortcut_label">URL: <input type="text" id="shortcut_url" value="${element_obj.link}"></label>
			<label class="shortcut_label">Image: <input type="text" id="shortcut_image" value="${element_obj.logo}"></label>
		`;
		var div = document.createElement("div");
		div.innerHTML = html;

		var button = document.createElement("button");
		button.innerHTML = html_button;
		button.addEventListener("click", () => {
			// create new element obj
			var name = document.getElementById("shortcut_name").value;
			var url = document.getElementById("shortcut_url").value;
			var image = document.getElementById("shortcut_image").value;
			var new_element_obj = {name:name,link:url,logo:image};

			// insert new link element
			var link = Shortcut.loadElement(new_element_obj);

			// remove old link element
			if (edit_element) {
				container.insertBefore(link, element);
				element.remove();
			}
			else container.append(link);

			saveUserData(new_element_obj, wrapper_index, element_index, "Shortcut", data, edit_element);
			dialog.close();
		});

		dialog.appendChild(div);
		dialog.appendChild(button);
	}

	static startEdit(wrapper) {
		var container = wrapper.querySelector(".shortcut_container");
		editContainer(container);

		for (var i = 0; i < container.children.length; i++) {
			this.editElement(container.children[i]);
		}
	}

	static stopEdit(wrapper) {
		var container = wrapper.querySelector(".shortcut_container");
		finishEditContainer(container);

		for (var i = 0; i < container.children.length; i++) {
			finishEditElement(container.children[i]);
			container.children[i].removeEventListener("click", Shortcut.createElementHelper);
		}
	}
}