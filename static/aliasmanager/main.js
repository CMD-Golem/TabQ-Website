const filter_mailbox = document.getElementById("filter_mailbox");
const alias_mailbox = document.getElementById("alias_mailbox");
const mail_hosting_id = document.getElementById("mail_hosting_id");
const bearer = document.getElementById("bearer");

const alias_list = document.getElementById("alias_list");

async function init() {
	var active_aliases = new Set(getAlias());
	var all_aliases = getFilter();

	var aliases = all_aliases.map(alias => ({
		value: alias,
		active: active_aliases.has(alias)
	}));

	for (var i = 0; i < aliases.length; i++) {


		var element = array[i];
	}
}

init()

async function getCookieData() {
	var res = await fetch("/api/aliasmanager/data");

	if (res.status != 200) {
		console.error(res);
		return;
	}

	var response = await res.json();
	console.log(response)

	filter_mailbox.value = response.filter_mailbox;
	alias_mailbox.value = response.alias_mailbox;
	mail_hosting_id.value = response.mail_hosting_id;
	bearer.value = response.bearer;
}

async function updateCookieData() {
	var body = {
		filter_mailbox: filter_mailbox.value,
		alias_mailbox: alias_mailbox.value,
		mail_hosting_id: mail_hosting_id.value,
		bearer: bearer.value
	}

	var res = await fetch("/api/aliasmanager/data", {
		method: "POST",
		body: JSON.stringify(body)
	});

	if (res.status != 200) {
		console.error(res);
		return;
	}
}

async function getAlias() {
	var res = await fetch("/api/aliasmanager/alias");

	if (res.status != 200) {
		console.error(res);
		return;
	}

	var response = await res.json();
	return response.data.aliases
}

async function createAlias(alias) {
	var res = await fetch("/api/aliasmanager/alias", {
		method: "POST",
		body: `{"alias":"${alias}"}`
	});

	if (res.status != 200) {
		console.error(res);
		return;
	}

	var response = await res.json();
	console.log(response)
}

async function removeAlias(alias) {
	var res = await fetch("/api/aliasmanager/alias/" + alias, {
		method: "DELETE"
	});

	if (res.status != 200) {
		console.error(res);
		return;
	}

	var response = await res.json();
	console.log(response)
}

async function getFilter() {
	var res = await fetch("/api/aliasmanager/filter");

	if (res.status != 200) {
		console.error(res);
		return;
	}

	var response = await res.json();
	console.log(response)
	console.log(response.data.scripts)

	for (var i = 0; i < response.data.scripts.length; i++) {
		var script = response.data.scripts[i];

		if (script.name == "aliasmanagerfilter") {
			var alias_list = JSON.parse(script.content
				.replace('require ["fileinto"];\nif not address :is ["to", "cc"] ', '')
				.replace('\n    ', '')
				.replace('\n] {\n    fileinto "Spam";\n}\r\n\r\n', ']')
			);
			break;
		}
		else var alias_list = [];
	}

	return alias_list;
}

async function updateFilter() {



	var res = await fetch("/api/aliasmanager/filter", {
		method: "PATCH",
		body: JSON.stringify(filter)
	});

	if (res.status != 200) {
		console.error(res);
		return;
	}

	var response = await res.json();
	console.log(response)
}