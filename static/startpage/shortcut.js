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

class Shortcut {
	static default_obj = {type:"Shortcut", styles:{cols:4, backgroundColor:"#2d2d38"}, content:[]};
	static label_name = "Add Shortcut";

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
		container.setAttribute("data-class", "Shortcut");
		container.classList.add("shortcut_container", "startpage_container");
		container.style.setProperty("--cols", element.styles.cols);
		container.style.setProperty("background-color", element.styles.backgroundColor);

		if (edit_mode_active) this.editContainer(container);
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

	static editContainer(element) {
		editContainer(element);
		var create_element = document.createElement("div");
		create_element.classList.add("shortcut_create", "shortcut_link");
		create_element.addEventListener("click", Shortcut.createElement);
		create_element.innerHTML = `<img src="img/add.svg"><p>Add Shortcut</p>`;

		element.appendChild(create_element);
	}

	static editElement(element) {
		editElement(element);
		element.addEventListener("click", Shortcut.createElement);
	}

	static createContainer(e) {
		
	}

	static createElement(e) {
		e.preventDefault();
		var target = e.currentTarget;
		var dialog = showDialog();
		var edit_element = false;

		var json = window.localStorage.getItem("user_data");
		var data = JSON.parse(json);

		var {container_index, element_index} = getPosition(target.parentElement, target);
		var container_obj = data.elements[container_index];

		// show current data if its not create button
		if (!target.classList.contains("shortcut_create")) {
			var element_obj = container_obj.content[element_index];
			edit_element = true;
		}
		else var element_obj = {name:"", link:"", logo:""};
		
		// create html
		var html = `
			<p>Add Shortcut</p>
			<label class="shortcut_label">Name: <input type="text" id="shortcut_name" value="${element_obj.name}"></label>
			<label class="shortcut_label">URL: <input type="text" id="shortcut_url" value="${element_obj.link}"></label>
			<label class="shortcut_label">Image: <input type="text" id="shortcut_image" value="${element_obj.logo}"></label>
		`;

		var button = document.createElement("button");
		button.innerHTML = "Create";
		button.addEventListener("click", () => {
			// create new element obj
			var name = document.getElementById("shortcut_name").value;
			var url = document.getElementById("shortcut_url").value;
			var image = document.getElementById("shortcut_image").value;
			var new_element_obj = {name:name,link:url,logo:image};

			// insert new link element
			var link = Shortcut.loadElement(new_element_obj);
			target.parentElement.insertBefore(link, target);

			// remove old link element
			if (edit_element) target.remove();

			saveUserData(new_element_obj, container_index, element_index, container_obj.type, data, edit_element);
			dialog.close();
		});

		dialog.innerHTML += html;
		dialog.appendChild(button);
	}

	static startEdit(element) {
		this.editContainer(element);

		for (var i = 0; i < element.children.length; i++) {
			this.editElement(element.children[i]);
		}
	}

	static stopEdit(element) {
		finishEditContainer(element);
		element.querySelector(".shortcut_create").remove();

		for (var i = 0; i < element.children.length; i++) {
			finishEditElement(element.children[i]);
			element.children[i].removeEventListener("click", Shortcut.createElement);
		}
	}
}