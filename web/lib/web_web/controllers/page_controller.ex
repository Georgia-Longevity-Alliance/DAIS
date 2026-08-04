defmodule WebWeb.PageController do
  use WebWeb, :controller

  def home(conn, _params) do
    conn
    |> put_resp_content_type("text/html")
    |> send_resp(200, """
    <!DOCTYPE html>
    <html data-theme="light">
    <head>
      <meta charset="utf-8">
      <meta name="viewport" content="width=device-width, initial-scale=1">
      <title>DAIS — Autonomous Intelligence Socket</title>
      <script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
      <link href="https://cdn.jsdelivr.net/npm/daisyui@5/dist/full.css" rel="stylesheet">
    </head>
    <body class="min-h-screen bg-base-200">
      <div class="hero min-h-screen">
        <div class="hero-content text-center">
          <div class="max-w-md">
            <h1 class="text-5xl font-bold mb-2">🛂 DAIS</h1>
            <p class="text-xl mb-1">Autonomous Intelligence Socket</p>
            <p class="text-sm opacity-70 mb-8">
              Open protocol for safe embodied AI.<br/>
              Give every device a passport, every intervention a trace.
            </p>
            <div class="space-y-3">
              <a href="/auth/google" class="btn btn-primary btn-wide">
                Sign in with Google
              </a>
              <div class="text-xs opacity-50">or</div>
              <a href="/passport/new" class="btn btn-outline btn-wide">
                Try without login
              </a>
            </div>
            <div class="mt-8 flex gap-4 justify-center text-xs opacity-50">
              <a href="https://github.com/Georgia-Longevity-Alliance/DAIS" class="link">GitHub</a>
              <span>Apache 2.0</span>
            </div>
          </div>
        </div>
      </div>
    </body>
    </html>
    """)
  end
end
