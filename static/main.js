var el_year = document.getElementById("year");
var el_week = document.getElementById("week");
var el_main = document.querySelector("main");

function load() {
	// show year selection
	var current_year = new Date().getFullYear();
	while (current_year - 2013 >= 0) {
		var element = document.createElement('option');
		element.value = current_year;
		element.innerHTML = current_year;
		el_year.appendChild(element);
		current_year--;
	}
}

load();

function getWeeks(year) {
	el_week.innerHTML = "";
	var date = new Date();

	// get current week
	if (year == date.getFullYear()) {
		date.setHours(0, 0, 0, 0);
		date.setDate(date.getDate() + 3 - (date.getDay() + 6) % 7);
		var week1 = new Date(date.getFullYear(), 0, 4);
		var current_week = 1 + Math.round(((date.getTime() - week1.getTime()) / 86400000 - 3 + (week1.getDay() + 6) % 7) / 7);
	}
	// get amount of weeks per year
	else {
		var day1 = new Date(year, 0, 1);
		var leap_year = new Date(year, 1, 29).getMonth() == 1;
		var current_week = day1.getDay() == 4 || leap_year && day1.getDay() == 3 ? 53 : 52;
	}
	
	// show week selection
	while (current_week > 0) {
		var element = document.createElement('option');
		element.value = current_week;
		element.innerHTML = current_week;
		el_week.appendChild(element);
		current_week--;
	}
}

async function loadCoop() {
	el_main.innerHTML = "";
	var year = el_year.value;
	var week = el_week.value;
	var fetch_amount = 5;

	// get publication date
    var date = new Date(year, 0, 4);
	date.setDate(date.getDate() - (date.getDay() || 7) + 1); // go to monday of first week
	date.setDate(date.getDate() + (week - 1) *7 + fetch_amount * 4); // monday of selected week + half of fetched amount

	var publication = await findDate(date, week, fetch_amount, 0);
	if (publication == undefined) return;

	// fetch  and show images
	var magazin_load = await fetch("/api/1/pages", {
		method: "POST",
		body: `{"date":"${publication}"}`,
	});

	if (magazin_load.status != 200) {
		console.error(magazin_load);
		return;
	}

	var magazin = await magazin_load.json();

	for (var i = 0; i < magazin.length; i++) {
		var element = document.createElement('img');
		element.src = magazin[i];
		el_main.appendChild(element);
	}
}

async function findDate(date, week, fetch_amount, loop_fix) {
	loop_fix += 1;
	// fetch data
	var fetch_date = new Date(date);

	var month_number = fetch_date.getMonth();
	if (month_number <= 9) var month = "0" + (month_number + 1).toString();
	else var month = month_number.toString();

	var day_number = fetch_date.getDate();
	if (day_number <= 9) var day = "0" + day_number.toString();
	else var day = day_number.toString();

	var publications_load = await fetch("/api/1/publications", {
		method: "POST",
		body: `{"date":"${fetch_date.getFullYear()}-${month}-${day}", "amount":${fetch_amount}}`,
	});

	if (publications_load.status != 200) {
		console.error(publications_load);
		return;
	}

	var pub = await publications_load.json();

	// check if seeked week exists
	var needs_newer = undefined;
	var publication = undefined

	for (var i = 0; i < pub.length; i++) {
		if (pub[i].edition_number == week) {
			publication = pub[i].publication_date;
			break;
		}
		else if (pub[i].edition_number < week) needs_newer = true;
		else needs_newer = false;
	}

	console.log(loop_fix)

	// return if found
	if (publication != undefined) return publication;
	// prevent to much requests
	else if (loop_fix >= 3) return;
	// fetch again if not, dependend if it needs to be newer or not
	else if (needs_newer) findDate(fetch_date.setDate(fetch_date.getDate() + fetch_amount * 7), week, fetch_amount, loop_fix);
	else findDate(fetch_date.setDate(fetch_date.getDate() - fetch_amount * 7), week, fetch_amount, loop_fix);
}


async function loadMigros() {
	// 	List with links to images of all pages: https://reader3.isu.pub/m-magazin/migros-magazin-45-2024-d-os/reader3_4.json
	el_main.innerHTML = "";
	var magazin_load = await fetch(`https://reader3.isu.pub/m-magazin/migros-magazin-${el_week.value}-${el_year.value}-d-os/reader3_4.json`);

	if (magazin_load.status != 200) {
		console.error(magazin_load);
		return;
	}

	var magazin = await magazin_load.json();

	for (var i = 0; i < magazin.document.pages.length; i++) {
		var element = document.createElement('img');
		element.src = "https://" + magazin.document.pages[i].imageUri;
		el_main.appendChild(element);
	}
}
