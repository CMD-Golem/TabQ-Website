var dragging_clone;
var pointer_pos = {x:0, y:0, ox:0, oy:0};
var target_store = {};

function dragStart(e) {
	var target = e.currentTarget;
	body.classList.add("started_dragging");

	var startpage_container = document.querySelectorAll(".startpage_container");
	target_store.index = Array.from(target.parentNode.children).indexOf(target);
	target_store.old_parent_id = Array.from(startpage_container).indexOf(target.parentElement);

	// create dragging clone
	dragging_clone = target.cloneNode(true);
	document.body.appendChild(dragging_clone);
	dragging_clone.classList.add("dragging_clone");

	var bounding_rect = target.getBoundingClientRect();

	dragging_clone.style.width = bounding_rect.width + "px";
	dragging_clone.style.height = bounding_rect.height + "px";

	if (e.dataTransfer != undefined) {
		e.dataTransfer.setDragImage(dragging_clone, 0, 0);
	}
	else if (e.touches != undefined) {
		pointer_pos.ox = e.touches[0].clientX - bounding_rect.left;
		pointer_pos.oy = e.touches[0].clientY - bounding_rect.top;
		dragging_clone.style.opacity = 0.7;
	}

	// fixe chromium bug
	setTimeout((target) => {
		target.classList.add("dragging");
	}, 0, target);
}

function dragEnd(e) {
	// handle chromium bugs
	if (e.currentTarget != null) var target = e.currentTarget;
	else var target = document.querySelector(".dragging");

	if (target == null || !target.classList.contains("dragging")) {
		setTimeout(dragEnd, 100, e);
		return;
	}

	// clean up
	target.classList.remove("dragging");
	body.classList.remove("started_dragging");
	dragging_clone.remove();

	// delete empty groups /* Doesnt work yet */
	var drag_containers = document.querySelectorAll(".drag_container");
	for (var i = 0; i < drag_containers.length; i++) {
		if (drag_containers[i].children.length == 0 && target.parentElement.id != "delete_element" && !target.parentElement.classList.contains("drag_create_container")) {
			drag_containers[i].remove();
			drag_containers[i].nextElementSibling.remove();
		}
	}

	// custom handling
	var json = window.localStorage.getItem("user_data");
	var data = JSON.parse(json);

	if (target.parentElement.id == "delete_element") {
		// delete element
		target.remove();
		changeContainerData(data, target_store, null, 0);
		return;
	}
	else if (target.parentElement.classList.contains("drag_create_container")) {
		// create new container
		var new_parent_obj = {type:"shortcuts", styles:{cols:4, backgroundColor:"#2d2d38"}, content:[]};

		var {container, spacer} = createContainer(new_parent_obj);
		target.remove();
		body.insertBefore(container, target.parentElement);
		body.insertBefore(spacer, container.previousElementSibling);
		container.append(target);

		var startpage_container = document.querySelectorAll(".startpage_container");
		var new_container_index = Array.from(startpage_container).indexOf(container);
		data.elements.splice(new_container_index, 0, new_parent_obj);

		console.log(new_container_index)

		changeContainerData(data, target_store, new_parent_obj, 0);
	}
	else {
		// store new position in local storage
		var startpage_container = document.querySelectorAll(".startpage_container");
		var new_parent_index = Array.from(startpage_container).indexOf(target.parentElement);
		var new_index = Array.from(target.parentElement.children).indexOf(target);
		var new_parent_obj = data.elements[new_parent_index];

		changeContainerData(data, target_store, new_parent_obj, new_index);
	}

	// animation
	var elements = target.parentElement.querySelectorAll(".draggable_element:not(.dragging)");
	var element_positions = new Map();

	for (var i = 0; i < elements.length; i++) {
		var bounding_rect = elements[i].getBoundingClientRect();
		element_positions.set(elements[i], bounding_rect);
	}

	animation(element_positions, elements);
}

async function dragOver(e) {
	e.preventDefault();
	
	// only run if position changed
	if (Math.abs(pointer_pos.x - e.clientX) > 5 || Math.abs(pointer_pos.y - e.clientY) > 5) {
		pointer_pos.x = e.clientX;
		pointer_pos.y = e.clientY;

		var container = e.currentTarget;
	}
	else if (e.touches != undefined && (Math.abs(pointer_pos.x - e.touches[0].clientX) > 5 || Math.abs(pointer_pos.y - e.touches[0].clientY) > 5)) {
		pointer_pos.x = e.touches[0].clientX;
		pointer_pos.y = e.touches[0].clientY;

		var container = document.elementFromPoint(pointer_pos.x, pointer_pos.y).closest(".drag_container");
		if (container == null) return;

		// move clone
		dragging_clone.style.left = pointer_pos.x - pointer_pos.ox + "px";
		dragging_clone.style.top = pointer_pos.y - pointer_pos.oy + "px";
	}
	else return;

	// get data
	var dragging = document.querySelector(".dragging");
	var not_dragging = container.querySelectorAll(".draggable_element:not(.dragging)");
	var element_positions = new Map();

	// append directly if no other elements in container
	// append it for container creation and elemente delete area
	if (not_dragging.length == 0 || container.id == "delete_element" || container.classList.contains("drag_create_container")) {
		container.appendChild(dragging);
		return;
	}

	// group all elements on the same row
	var container_rows = [];
	for (var i = 0; i < not_dragging.length; i++) {
		var bounding_rect = not_dragging[i].getBoundingClientRect();
		element_positions.set(not_dragging[i], bounding_rect);

		getRows(not_dragging[i], container_rows, true);
	}

	// add row of target
	getRows(dragging, container_rows, false);

	// find closest row
	var closest_row = undefined;
	var closest_value = Infinity;

	for (var i = 0; i < container_rows.length; i++) {
		var value = Math.abs(pointer_pos.y + document.documentElement.scrollTop - container_rows[i].y);
		if (value < closest_value) {
			closest_value = value;
			closest_row = container_rows[i].elements;
		}
	}

	// find closest element in row
	var closest_element = undefined;
	closest_value = Infinity;
	
	for (var i = 0; i < closest_row.length; i++) {
		var value = pointer_pos.x - closest_row[i].x;
		if (Math.abs(value) < closest_value && value < 0) {
			closest_value = Math.abs(value);
			closest_element = closest_row[i].el;
		}
	}

	// if no element could be found right from the pointer use last element in row
	var last_element = closest_row[closest_row.length - 1].el;

	if (closest_element) container.insertBefore(dragging, closest_element);
	else if (last_element.nextSibling) container.insertBefore(dragging, last_element.nextSibling);
	else container.appendChild(dragging);

	// animation
	animation(element_positions, not_dragging);
}

function animation(position, elements) {
	for (var i = 0; i < elements.length; i++) {
		var element = elements[i];

		var old_position = position.get(element);
		var new_position = element.getBoundingClientRect();

		var dx = old_position.left - new_position.left;
		var dy = old_position.top - new_position.top;

		if (dx || dy) element.animate([{transform:`translate(${dx}px, ${dy}px)`}, {transform:"translate(0, 0)"}], {duration:200, easing:"ease-in-out"});
	}
}

function getRows(element, container_rows, add) {
	var bounding_rect = element.getBoundingClientRect();
	var vertical_center = element.offsetTop + bounding_rect.height / 2;
	var horizontal_position = element.offsetLeft + bounding_rect.width /2;

	// create new row if it doesnt exist yet
	var existing_row = container_rows.find(row => Math.abs(row.y - vertical_center) < 5);

	if (!existing_row) {
		existing_row = {y:vertical_center, elements:[]};
		container_rows.push(existing_row);
	}

	// add info to row
	if (add) existing_row.elements.push({el:element, x:horizontal_position});
}