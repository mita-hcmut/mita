const BASE_URL =
  "https://e-learning.hcmut.edu.vn/webservice/rest/server.php?moodlewsrestformat=json";

const res = await fetch(BASE_URL, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    wstoken: "YOUR_TOKEN",
    wsfunction: "core_webservice_get_site_info",
  }),
});

const body = await res.json();
