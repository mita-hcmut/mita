const OAUTH2_ADDR = "localhost:8443/default";

async function get_code() {
  const conn = Deno.listen({ port: 0 });
  if (conn.addr.transport !== "tcp") throw new Error("Expected TCP connection");

  const oneshot_server = run_oneshot_server(conn);

  let url = new URL(`http://${OAUTH2_ADDR}/authorize`);
  url.searchParams.set("client_id", "client_id");
  url.searchParams.set("response_type", "code");
  url.searchParams.set("scope", "openid");
  url.searchParams.set("state", "state");
  url.searchParams.set("nonce", "nonce");
  url.searchParams.set(
    "redirect_uri",
    `http://localhost:${conn.addr.port}`,
  );

  let data = new URLSearchParams();
  data.set("username", "khang");
  data.set("claims", "");

  await fetch(url, {
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    method: "POST",
    body: data,
  });

  const code = await oneshot_server;

  url = new URL(`http://${OAUTH2_ADDR}/token`);
  data = new URLSearchParams();
  data.set("client_id", "client_id");
  data.set("code", code);
  data.set("redirect_uri", `http://localhost:${conn.addr.port}`);
  data.set("grant_type", "authorization_code");

  const res = await fetch(url, {
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    method: "POST",
    body: data,
  });

  const { id_token } = await res.json();

  return id_token as string;
}

async function run_oneshot_server(conn: Deno.Listener): Promise<string> {
  try {
    const httpConn = Deno.serveHttp(await conn.accept());
    const { request, respondWith } = (await httpConn.nextRequest())!;
    const url = new URL(request.url);
    const code = url.searchParams.get("code")!;
    await respondWith(new Response("OK"));
    return code;
  } finally {
    conn.close();
  }
}

const id_token = await get_code();

const res = await fetch("http://localhost:8080/token", {
  method: "PUT",
  headers: {
    "Authorization": `Bearer ${id_token}`,
    "Content-Type": "application/x-www-form-urlencoded",
  },
  body: new URLSearchParams({ moodle_token: "a".repeat(32) }),
});

console.log(res.status);

let body = await res.text();

console.log("end")
