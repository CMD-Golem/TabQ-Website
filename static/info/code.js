document.getElementById("footer_date").innerHTML = new Date().getFullYear();

// spoilers
function toggleDetail(e, close_others) {
	var spoilers = document.getElementsByTagName("details");
	
	if (e.target) {
		var el = e.target.parentElement;
		e.stopPropagation();
		if (e.target.nodeName == "DETAILS") { e.preventDefault(); }
		if (e.target.nodeName == "SUMMARY") { e.preventDefault(); }
		if (e.target.nodeName != "SUMMARY") { return; }
	}
	else { var el = e; }

	if (close_others == undefined) { close_others = false; }
	if (!el.classList.contains("active")) { var spoiler_active = true; }
	if (close_others) {
		for (var i = 0; i < spoilers.length; i++) {
			spoilers[i].classList.remove("active");
			spoilers[i].style.maxHeight = "44px";
		}
	}

	if (spoiler_active) {
		el.classList.add("active");
		el.style.maxHeight = el.scrollHeight + 35 + "px";
	}
	else if (!close_others) {
		el.classList.remove("active");
		el.style.maxHeight = "44px";
	}
}

// Textarea
var el_textarea = document.getElementsByTagName("textarea");
for (var i = 0; i < el_textarea.length; i++) {
	el_textarea[i].addEventListener("input", (e) => {
		e.target.style.height = "auto";
		e.target.style.height = e.target.scrollHeight + 12 + "px";
	});
}


// send Form
async function sendForm() {
	var el_mains = document.getElementsByTagName("main");
	var el_description = document.getElementById("description");

	if (el_description.value == "") return alert("Add your comment first");

	var form_body = {
		subject: "TabQ Contact",
		body: `
			<p>Email: ${document.getElementById("email").value}</p>
			<p>${el_description.value}</p>`
	};

	var response = await fetch("https://api.tabq.ch/forms-fg/mail", {
		method: "POST",
		body: JSON.stringify(form_body),
	});

	if (response.ok) {
		el_mains[0].style.display = "none";
		el_mains[1].style.display = "block";
	}
	else {
		var error = await response.text();
		console.error(error);
		alert("An error has occurred: " + error);
	}
}

function goBack() {
	var history_length = history.length;
	if (history_length >= 2) window.history.go(-1);
	else window.location.href = "/";
}

// #####################################################################
// Support button iframe
function vposFrame() {
	// https://docs.payrexx.com/developer/guides/embedding/iframe
	// create iframe
	var iframe = document.createElement("iframe");
	iframe.allow = "payment *";
	iframe.src = "https://tabq.payrexx.com/de/vpos?appview=1&purpose=cmdgolem";

	iframe.addEventListener("load", () => {
		container.classList.remove("vpos_loading");
		iframe.contentWindow.postMessage(
			JSON.stringify({ origin: window.location.origin }),
			iframe.src
		);
	});

	// create container
	var container  = document.createElement("div");
	container.classList.add("vpos_container", "vpos_loading");
	container.appendChild(iframe);
	document.querySelector("body").appendChild(container);

	// listen to iframe messages
	window.addEventListener("message", e => {
		if (typeof e.data != "string") return;
		var data = JSON.parse(e.data);

		// remove iframe
		if (data.payrexx?.closeModal == "") container.remove();

		// change size of iframe
		else if (typeof data.payrexx?.height == "string") {
			var height = data.payrexx.height;
			if (parseInt(height) > 800) iframe.style.height = height;
			else iframe.style.height = "800px"
		}
	});
}