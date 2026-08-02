var aliases_object = [];

async function init() {
	// get cookie data
	var cookie_not_set = await getCookieData();

	if (cookie_not_set) {
		document.getElementById("settings").showModal();
		return;
	}

	// build aliases
	var active_aliases = new Set(await getAlias());
	var all_aliases = await getFilter();

	aliases_object = all_aliases.map((mail) => {
		if (mail.endsWith(alias_domain_value)) {
			var alias = mail.replace(alias_domain_value, "");
			var active = active_aliases.has(alias);
		}
		else {
			var alias = false;
			var active = false;
		}

		return {
			id: mail.replace("@", ""),
			mail: mail,
			alias: alias,
			active: active
		};
	});

	loadHtml();
}

const alias_list = document.getElementById("alias_list");
const searchbox = document.getElementById("searchbox")

function loadHtml() {
	alias_list.innerHTML = "";
	searchbox.value = "";

	for (var i = 0; i < aliases_object.length; i++) {
		var alias = aliases_object[i];
		alias.index = i;

		var alias_el = document.createElement("alias");
		alias_el.setAttribute("data-index", i);
		alias_el.id = alias.id;

		if (alias.active) var toggle = " checked";
		else if (alias.alias === false) var toggle = " disabled";
		else var toggle = "";

		alias_el.innerHTML = `<span>${alias.mail}</span>
			<button title="copy" class="alias_copy" onclick="copy(this.previousElementSibling.innerText)"></button>
			<button title="delete" class="alias_delete" onclick="deleteAlias(this)"></button>
			<div class="toggle">
				<input type="checkbox" name="alias_toggle" onchange="toggleAlias(this)" title="toggle active"${toggle}>
				<label for="alias_toggle"></label>
			</div>`;
		
		alias_list.appendChild(alias_el);
	}
}

init();

const alias_text = document.getElementById("alias_text");

function startCreateAlias() {
	alias_text.innerText = alias_domain_value;
	document.getElementById("create_alias").showModal();

	alias_text.focus();
}

function createAlias() {
	document.getElementById("create_alias").close();

	var mail = alias_text.innerText.toLowerCase();
	var mail_array = mail.split("@");

	if (mail_array.length !== 2) {
		error.innerText = "Alias must include one @ symbol";
		error.parentElement.showModal();
		return;
	}

	var alias = mail_array[0];
	var domain = mail_array[1];
	var id = alias + domain;
	
	if (!mail.endsWith(alias_domain_value)) var alias = false;

	aliases_object.push({
		id: id,
		mail: mail,
		alias: alias,
		active: false,
	});

	aliases_object.sort((a, b) => a.mail.localeCompare(b.mail));

	loadHtml();
	updateFilter();

	var alias_el = document.getElementById(id)
	alias_el.classList.add("highlight");
	alias_el.scrollIntoView({block: "center", behavior: "smooth"});
	setTimeout(function(){ alias_el.classList.remove("highlight"); }, 800);
}

function generateRandomString() {
	var letter_count = 20;

	var random_string = Array.from(crypto.getRandomValues(new Uint8Array(letter_count)))
		.map(x => "abcdefghijklmnopqrstuvwxyz0123456789"[x % 36])
		.join("");

	var mail =  alias_text.innerText.toLowerCase().split("@");
	alias_text.innerText = `${mail[0]}-${random_string}@${mail[1]}`;
}

function toggleAlias(el) {
	var index = el.parentElement.parentElement.getAttribute('data-index');
	var alias = aliases_object[index];
	alias.active = el.checked;

	if (alias.active) activateAlias(alias.alias);
	else deactivateAlias(alias.alias);
}

function deleteAlias(el) {
	var index = el.parentElement.getAttribute('data-index');
	var alias = aliases_object[index];

	if (alias.active) deactivateAlias(alias.alias);
	aliases_object.splice(alias.index, 1);

	loadHtml();
	updateFilter();
}

function search(value) {
	for (var i = 0; i < alias_list.children.length; i++) {
		var alias_el = alias_list.children[i];
		if (!alias_el.firstElementChild.innerText.includes(value)) {
			alias_el.classList.add("search_hidden");
		}
		else alias_el.classList.remove("search_hidden");
	}
}

function clearSearch() {
	for (var i = 0; i < alias_list.children.length; i++) {
		alias_list.children[i].classList.remove("search_hidden");
	}

	searchbox.value = "";
}

function copy(text) {
	var copy = document.createElement("textarea");
	copy.value = text;
	document.body.appendChild(copy);
	copy.select();
	document.execCommand("copy");
	copy.remove();
}