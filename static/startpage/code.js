var body = document.querySelector("body");
var edit_mode_active = false;

const classMap = {
	Shortcut
};

function loadData() {
	var json = window.localStorage.getItem("user_data");
	if (json == null) return;

	var data = JSON.parse(json);

	// apply styles
	var root = document.querySelector(":root")
	root.style.setProperty("--backgroundColor", data.style.backgroundColor);
	root.style.setProperty("--foregroundColor", data.style.foregroundColor);

	// init delete element
	var delete_element = document.getElementById("delete_element");
	delete_element.addEventListener("dragover", dragOver);
	delete_element.addEventListener("touchmove", dragOver);

	// add styles and init
	for (var [_, value] of Object.entries(classMap)) value.init();

	// insert elements
	for (var i = 0; i < data.elements.length; i++) {
		var defiend_class = classMap[data.elements[i].type];
		var container = defiend_class.load(data.elements[i]);
		var spacer = createSpacer();

		body.appendChild(container);
		body.appendChild(spacer);
	}
}
loadData();

function createSpacer() {
	var spacer = document.createElement("div");
	spacer.classList.add("spacer", "drag_create_container");

	if (edit_mode_active) editSpacer(spacer);
	return spacer;
}

function editSpacer(element) {
	element.classList.add("drag_container");
	element.addEventListener("dragover", dragOver);
	element.addEventListener("touchmove", dragOver);
}

// ##################################################
// edit function
function startEdit() {
	document.getElementById("general_menu").style.display = "none";
	document.getElementById("edit_menu").style.display = "block";
	
	var spacers = document.querySelectorAll(".spacer");
	for (var i = 0; i < spacers.length; i++) {
		editSpacer(spacers[i]);
	}

	var startpage_containers = document.querySelectorAll(".startpage_container");
	for (var i = 0; i < startpage_containers.length; i++) {
		var class_name = startpage_containers[i].getAttribute("data-class");
		var defiend_class = classMap[class_name];
		defiend_class.startEdit(startpage_containers[i]);
	}
}

function stopEdit() {
	document.getElementById("general_menu").style.display = "block";
	document.getElementById("edit_menu").style.display = "none";

	var spacers = document.querySelectorAll(".spacer");
	for (var i = 0; i < spacers.length; i++) {
		spacers[i].classList.remove("drag_container");
		spacers[i].removeEventListener("dragover", dragOver);
		spacers[i].removeEventListener("touchmove", dragOver);
	}

	var startpage_containers = document.querySelectorAll(".startpage_container");
	for (var i = 0; i < startpage_containers.length; i++) {
		var class_name = startpage_containers[i].getAttribute("data-class");
		var defiend_class = classMap[class_name];
		defiend_class.stopEdit(startpage_containers[i]);
	}
}

function changeContainerData(data, target_store, new_container_index, new_element_index) {
	var old_container_obj = data.elements[target_store.container_index];
	var element_obj = old_container_obj.content[target_store.element_index];

	// remove element from old container
	old_container_obj.content.splice(target_store.element_index, 1);

	// remove empty container
	if (old_container_obj.content.length == 0) data.elements.splice(target_store.container_index, 1);

	// get new container
	var new_container_obj = data.elements[new_container_index];

	// create new container
	if (new_element_index == null) {
		var defiend_class = classMap[old_container_obj.type];

		new_container_obj = structuredClone(defiend_class.default_obj);
		data.elements.splice(new_container_index, 0, new_container_obj);
		new_element_index = 0;
	}
	// add element to new container if it exists
	if (new_container_index != null) {
		new_container_obj.content.splice(new_element_index, 0, element_obj);
	}

	window.localStorage.setItem("user_data", JSON.stringify(data));
}

function newItem() {
	dialog.showModal();

	for (var [_, value] of Object.entries(classMap)) {
		
	}

	dialog.innerHTML
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