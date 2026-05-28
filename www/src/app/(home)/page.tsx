import Link from 'next/link';

const engines = [
  { name: 'SpiderMonkey', note: 'fast bootstrap, unstable teardown' },
  { name: 'JavaScriptCore', note: 'clean host callbacks, shared jstd surface' },
  { name: 'V8', note: 'stable default runner, fresh isolate per run' },
];

const features = [
  'One script, multiple engines',
  'Shared jstd bootstrap',
  'Color-formatted runtime errors',
  'Docs-first developer flow',
];

export default function HomePage() {
  return (
    <main className="relative overflow-hidden">
      <div className="absolute inset-0 pointer-events-none">
        <div className="hero-spin absolute left-[-8rem] top-20 h-72 w-72 rounded-full border border-[var(--line)] opacity-60" />
        <div className="absolute right-[-5rem] top-32 h-56 w-56 rounded-full bg-[radial-gradient(circle,_rgba(255,179,107,0.55),_transparent_70%)] blur-2xl" />
        <div className="absolute bottom-0 left-1/2 h-64 w-[42rem] -translate-x-1/2 bg-[radial-gradient(circle,_rgba(215,79,42,0.16),_transparent_68%)] blur-3xl" />
      </div>

      <section className="relative mx-auto flex min-h-[calc(100vh-4rem)] max-w-7xl flex-col justify-center px-6 py-16 sm:px-10 lg:px-16">
        <div className="grid items-center gap-14 lg:grid-cols-[1.2fr_0.8fr]">
          <div className="hero-float">
            <div className="mb-5 inline-flex items-center gap-3 rounded-full border border-[var(--line)] bg-[var(--panel)] px-4 py-2 text-xs uppercase tracking-[0.28em] text-[var(--muted)] backdrop-blur">
              <span className="h-2 w-2 rounded-full bg-[var(--accent)]" />
              experimental javascript runner
            </div>

            <h1 className="max-w-4xl text-5xl font-semibold leading-[0.94] tracking-[-0.06em] text-[var(--text)] sm:text-6xl lg:text-8xl">
              Run the same script on wildly different engines.
            </h1>

            <p className="mt-6 max-w-2xl text-base leading-8 text-[var(--muted)] sm:text-lg">
              <span className="font-medium text-[var(--text)]">o-</span> is a
              tiny multi-engine JavaScript runner with a shared bootstrap
              layer. It lets you compare SpiderMonkey, JavaScriptCore, and V8
              behind one interface without flattening everything into the same
              boring developer experience.
            </p>

            <div className="hero-float-delay mt-10 flex flex-wrap items-center gap-4">
              <Link
                href="/docs/getting-started"
                className="rounded-full bg-[var(--text)] px-6 py-3 text-sm font-medium text-white transition-transform hover:-translate-y-0.5"
              >
                Open the docs
              </Link>
              <Link
                href="/docs/engines"
                className="rounded-full border border-[var(--line)] bg-[var(--panel)] px-6 py-3 text-sm font-medium text-[var(--text)] backdrop-blur transition-transform hover:-translate-y-0.5"
              >
                Compare engines
              </Link>
            </div>

            <div className="mt-12 flex flex-wrap gap-3">
              {features.map((feature) => (
                <span
                  key={feature}
                  className="rounded-full border border-[var(--line)] bg-white/55 px-4 py-2 text-sm text-[var(--muted)] backdrop-blur"
                >
                  {feature}
                </span>
              ))}
            </div>
          </div>

          <div className="hero-float-delay relative">
            <div className="rounded-[2rem] border border-[var(--line)] bg-[var(--panel)] p-5 shadow-[0_24px_90px_rgba(62,39,20,0.08)] backdrop-blur-xl">
              <div className="rounded-[1.5rem] border border-[var(--line)] bg-[#1f1712] p-5 text-left text-sm text-[#f5e8d6] shadow-inner">
                <div className="mb-4 flex items-center gap-2">
                  <span className="h-3 w-3 rounded-full bg-[#ff845b]" />
                  <span className="h-3 w-3 rounded-full bg-[#ffc163]" />
                  <span className="h-3 w-3 rounded-full bg-[#65d48d]" />
                </div>
                <pre className="overflow-x-auto text-xs leading-7 sm:text-sm">
                  <code>{`$ o- run index.js
12586269025

$ cat ~/.config/o-/config.toml
[toolchain]
name = "spidermonkey"

$ o- toolchain add spidermonkey
$ o- toolchain add javascriptcore

Execution Error: JavaScript exception is pending
--> index.js
 | console.log(fib(50))`}</code>
                </pre>
              </div>

              <div className="mt-5 grid gap-3">
                {engines.map((engine) => (
                  <div
                    key={engine.name}
                    className="rounded-2xl border border-[var(--line)] bg-white/60 px-4 py-4 backdrop-blur"
                  >
                    <div className="flex items-center justify-between gap-4">
                      <h2 className="text-lg font-medium text-[var(--text)]">
                        {engine.name}
                      </h2>
                      <span className="rounded-full bg-[var(--accent)]/10 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-[var(--accent)]">
                        backend
                      </span>
                    </div>
                    <p className="mt-2 text-sm leading-6 text-[var(--muted)]">
                      {engine.note}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  );
}
