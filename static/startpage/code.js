var body = document.querySelector("body");
var container_list = document.getElementById("container_list");
var edit_mode_active = false;

const classMap = {
	Shortcut
};

function loadData() {
	var json = window.localStorage.getItem(location.pathname);
	if (json == null) {
		json = '{"style":{"backgroundColor":"#131319", "foregroundColor":"#2d2d38"},"elements":[]}';
		startEdit();
	};

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
	var spacer = createSpacer();
	container_list.appendChild(spacer);

	for (var i = 0; i < data.elements.length; i++) {
		var defiend_class = classMap[data.elements[i].type];
		var container = defiend_class.load(data.elements[i]);
		var spacer = createSpacer();

		container_list.appendChild(container.parentElement);
		container_list.appendChild(spacer);
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
	element.addEventListener("click", showContainerMenu);
}

// ##################################################
// handle element changes
function createContainer(type, used_spacer, element) {
	var container = classMap[type].loadContainer();
	var spacer = createSpacer();

	if (used_spacer.isConnected) container_list.insertBefore(container.parentElement, used_spacer.nextElementSibling);
	else container_list.insertBefore(container.parentElement, container_list.children[0]);
	
	if (container.parentElement.nextElementSibling == null) container_list.append(spacer);
	else container_list.insertBefore(spacer, container.parentElement.nextElementSibling);
	

	if (element == null) classMap[type].createElement(container, undefined, true);
	else {
		element.remove();
		container.appendChild(element);
	}

	return container;
}

function getPosition(container, element) {
	var startpage_wrappers = document.querySelectorAll(".startpage_wrapper");
	var wrapper_index = Array.from(startpage_wrappers).indexOf(container.parentElement);

	if (element == undefined) return {wrapper_index, element_index:null};
	
	var element_index = Array.from(container.children).indexOf(element);
	return {wrapper_index, element_index};
}

function changeContainerData(data, target_store, new_wrapper_index, new_element_index) {
	var old_container_obj = data.elements[target_store.wrapper_index];
	var element_obj = old_container_obj.content[target_store.element_index];

	// remove element from old container
	old_container_obj.content.splice(target_store.element_index, 1);

	// remove empty container
	if (old_container_obj.content.length == 0) data.elements.splice(target_store.wrapper_index, 1);

	saveUserData(element_obj, new_wrapper_index, new_element_index, old_container_obj.type, data, false);
}

function saveUserData(element_obj, wrapper_index, element_index, type, data, replace_element) {
	// get user data if it isnt delivered
	if (data == null) {
		var json = window.localStorage.getItem(location.pathname);
		data = JSON.parse(json);
	}

	// get new container
	var container_obj = data.elements[wrapper_index];

	// create new container
	if (element_index == null) {
		container_obj = structuredClone(classMap[type].default_obj);
		data.elements.splice(wrapper_index, 0, container_obj);
		element_index = 0;
	}
	// add/replace element in container if it exists
	if (replace_element) var splice_length = 1;
	else var splice_length = 0;

	if (wrapper_index != null) {
		container_obj.content.splice(element_index, splice_length, element_obj);
	}

	window.localStorage.setItem(location.pathname, JSON.stringify(data));
}

// ##################################################
// edit function
function startEdit() {
	body.classList.add("edit_mode");
	edit_mode_active = true;
	
	var spacers = document.querySelectorAll(".spacer");
	for (var i = 0; i < spacers.length; i++) {
		editSpacer(spacers[i]);
	}

	var startpage_wrappers = document.querySelectorAll(".startpage_wrapper");
	for (var i = 0; i < startpage_wrappers.length; i++) {
		var class_name = startpage_wrappers[i].getAttribute("data-class");
		var defiend_class = classMap[class_name];
		defiend_class.startEdit(startpage_wrappers[i]);
	}
}

function stopEdit() {
	body.classList.remove("edit_mode");
	edit_mode_active = false;

	var spacers = document.querySelectorAll(".spacer");
	for (var i = 0; i < spacers.length; i++) {
		spacers[i].classList.remove("drag_container");
		spacers[i].removeEventListener("dragover", dragOver);
		spacers[i].removeEventListener("touchmove", dragOver);
	}

	var startpage_wrappers = document.querySelectorAll(".startpage_wrapper");
	for (var i = 0; i < startpage_wrappers.length; i++) {
		var class_name = startpage_wrappers[i].getAttribute("data-class");
		var defiend_class = classMap[class_name];
		defiend_class.stopEdit(startpage_wrappers[i]);
	}
}

function editContainer(container) {
	container.classList.add("drag_container");
	container.addEventListener("dragover", dragOver);
	container.addEventListener("touchmove", dragOver);
}

function editElement(element) {
	element.classList.add("draggable_element");
	element.addEventListener("dragstart", dragStart);
	element.addEventListener("dragend", dragEnd);
	element.addEventListener("touchstart", dragStart);
	element.addEventListener("touchend", dragEnd);
}

function finishEditContainer(element) {
	element.classList.remove("drag_container");
	element.removeEventListener("dragover", dragOver);
	element.removeEventListener("touchmove", dragOver);
}

function finishEditElement(element) {
	element.classList.remove("draggable_element");
	element.removeEventListener("dragstart", dragStart);
	element.removeEventListener("dragend", dragEnd);
	element.removeEventListener("touchstart", dragStart);
	element.removeEventListener("touchend", dragEnd);
}

function showContainerMenu(e) {
	var dialog = showDialog("any");
	var spacer = e.currentTarget;

	for (var [_, value] of Object.entries(classMap)) {
		var button = document.createElement("button");
		button.addEventListener("click", () => { createContainer(value.default_obj.type, spacer, null); });
		button.innerHTML = value.label_name;
		dialog.appendChild(button);
	}
}

function showDialog(closedby) {
	var dialog = document.querySelector("dialog");
	dialog.innerHTML = "";
	dialog.showModal();

	if (closedby == "needed") {
		dialog.setAttribute("closedby", "none");
		return dialog;
	}
	else dialog.setAttribute("closedby", closedby); // closedby any / none

	// add close button
	var button = document.createElement("button");
	button.innerHTML = "Close";
	button.addEventListener("click", e => {
		e.currentTarget.parentElement.close();
	});
	dialog.appendChild(button);

	return dialog;
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
		window.localStorage.setItem(location.pathname, readerEvent.target.result);
		document.location.reload();
	}
}

function exportData(name) {
	var link = document.createElement('a');
	link.download = name + ".json";
	link.href = "data:text/plain;charset=utf-8," + window.localStorage.getItem(location.pathname);
	link.click();
}