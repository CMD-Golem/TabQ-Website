var body = document.querySelector("body");

function loadData() {
	var json = window.localStorage.getItem("user_data");
	if (json == null) return;

	var data = JSON.parse(json);

	// apply styles
	body.style.backgroundColor = data.style.backgroundColor;

	// init delete element
	var delete_element = document.getElementById("delete_element");
	delete_element.addEventListener("dragover", dragOver);
	delete_element.addEventListener("touchmove", dragOver);

	// insert elements
	for (var i = 0; i < data.elements.length; i++) {
		if (data.elements[i].type == "shortcuts") insertShortcuts(data.elements[i]);
	}
}
loadData();

function insertShortcuts(element) {
	var {container, spacer} = createContainer(element);

	for (var i = 0; i < element.content.length; i++) {
		var link_data = element.content[i];

		var link = document.createElement("a");
		link.href = link_data.link;
		link.innerHTML = `<img src="${link_data.logo}"><p>${link_data.name}</p>`;
		link.classList.add("draggable_element");
		link.addEventListener("dragstart", dragStart);
		link.addEventListener("dragend", dragEnd);
		link.addEventListener("touchstart", dragStart);
		link.addEventListener("touchend", dragEnd);

		container.appendChild(link);
	}

	body.appendChild(container);
	body.appendChild(spacer);
}

function createContainer(element) {
	var container = document.createElement("div");
	container.classList.add("shortcut_container", "drag_container", "startpage_container");
	container.addEventListener("dragover", dragOver);
	container.addEventListener("touchmove", dragOver);
	container.style.setProperty("--cols", element.styles.cols);
	container.style.setProperty("background-color", element.styles.backgroundColor);

	var spacer = document.createElement("div");
	spacer.classList.add("shortcut_spacer", "drag_create_container");
	spacer.addEventListener("dragover", dragOver);
	spacer.addEventListener("touchmove", dragOver);

	return {container, spacer};
}

function changeContainerData(data, target_store, new_parent_obj, new_index) {
	var old_parent_obj = data.elements[target_store.old_parent_id];
	var obj = old_parent_obj.content[target_store.index];

	old_parent_obj.content.splice(target_store.index, 1);
	if (old_parent_obj.content.length == 0) data.elements.splice(target_store.old_parent_id, 1);
	new_parent_obj?.content.splice(new_index, 0, obj);

	window.localStorage.setItem("user_data", JSON.stringify(data));
}

// ##################################################
// import
var import_data = document.createElement('input');
import_data.type = 'file';
import_data.accept = '.json';

import_data.onchange = e => { 
	var reader = new FileReader();
	reader.readAsText(e.target.files[0],'UTF-8');

	reader.onload = readerEvent => {
		window.localStorage.setItem("user_data", readerEvent.target.result);
		document.location.reload();
	}
}

function exportData(name) {
	var link = document.createElement('a');
	link.download = name + ".json";
	link.href = "data:text/plain;charset=utf-8," + window.localStorage.getItem("user_data");;
	link.click();
}