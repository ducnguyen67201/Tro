import { useEffect, useState } from "react";
import { siteCopy, type DemoPhase, type Locale, type SiteCopy } from "./copy";

function usePrefersReducedMotion() {
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => setReduced(media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);

  return reduced;
}

function TroMark() {
  return (
    <span className="tro-mark" aria-hidden="true">
      <span />
      <span />
      <span />
    </span>
  );
}

function PointerIcon() {
  return (
    <svg viewBox="0 0 32 38" role="img" aria-label="Tro cursor">
      <path
        d="M3.25 2.4 27.8 23.1l-11.3 1.1-6.25 9.65L3.25 2.4Z"
        fill="currentColor"
        stroke="white"
        strokeWidth="2.25"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function MicrophoneIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 15.5a4 4 0 0 0 4-4v-5a4 4 0 0 0-8 0v5a4 4 0 0 0 4 4Z" />
      <path d="M5.5 11.5a6.5 6.5 0 0 0 13 0M12 18v3M9 21h6" />
    </svg>
  );
}

function WifiIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M3.4 8.7a13.7 13.7 0 0 1 17.2 0M6.6 12.1a8.6 8.6 0 0 1 10.8 0M9.8 15.5a3.5 3.5 0 0 1 4.4 0M12 19h.01" />
    </svg>
  );
}

function BatteryIcon() {
  return (
    <svg viewBox="0 0 30 16" aria-hidden="true">
      <rect x="1" y="2" width="24" height="12" rx="3" />
      <path d="M27 6v4" />
      <rect
        className="battery-fill"
        x="3.5"
        y="4.5"
        width="17"
        height="7"
        rx="1.5"
      />
    </svg>
  );
}

function MacControls() {
  return (
    <span className="mac-controls" aria-hidden="true">
      <i />
      <i />
      <i />
    </span>
  );
}

function ComputerUseDemo({ copy }: { copy: SiteCopy["demo"] }) {
  const reducedMotion = usePrefersReducedMotion();
  const [phase, setPhase] = useState<DemoPhase>("idle");
  const [run, setRun] = useState(0);
  const activePhase: DemoPhase = reducedMotion ? "solved" : phase;

  const replay = () => {
    setPhase("idle");
    setRun((value) => value + 1);
  };

  useEffect(() => {
    if (reducedMotion) return;

    const timers = [
      window.setTimeout(() => setPhase("targeting"), 350),
      window.setTimeout(() => setPhase("listening"), 2_250),
      window.setTimeout(() => setPhase("thinking"), 4_300),
      window.setTimeout(() => setPhase("solved"), 5_650),
      window.setTimeout(() => {
        setPhase("idle");
        setRun((value) => value + 1);
      }, 10_500),
    ];

    return () => timers.forEach((timer) => window.clearTimeout(timer));
  }, [reducedMotion, run]);

  return (
    <section className="demo-section" id="demo" aria-labelledby="demo-title">
      <div className="demo-heading">
        <div>
          <p className="section-label">{copy.label}</p>
          <h2 id="demo-title">{copy.title}</h2>
        </div>
        <div className="demo-status" aria-live="polite">
          <span className={`status-dot status-dot--${activePhase}`} />
          <span>{copy.statuses[activePhase]}</span>
          <button
            type="button"
            className="replay-button"
            onClick={replay}
            aria-label={copy.replayLabel}
          >
            {copy.replay}
          </button>
        </div>
      </div>

      <div className="computer-window" data-phase={activePhase}>
        <div className="window-bar">
          <div className="window-controls" aria-hidden="true">
            <span />
            <span />
            <span />
          </div>
          <div className="window-address">
            <span className="address-lock">●</span>
            {copy.address}
          </div>
          <span className="window-progress">{copy.progress}</span>
        </div>

        <div className="workspace">
          <div className="workspace-nav" aria-hidden="true">
            <span className="mini-brand">
              <TroMark />
            </span>
            <span className="nav-line nav-line--long" />
            <span className="nav-line" />
            <span className="nav-line" />
            <span className="nav-line nav-line--bottom" />
          </div>

          <article className="question-sheet">
            <div className="question-meta">
              <span>{copy.questionNumber}</span>
              <span>{copy.points}</span>
            </div>
            <p className="question-kicker">{copy.topic}</p>
            <h3>{copy.question}</h3>
            <code>f(x) = (x − 2)² + 1</code>
            <div className="answer-options" aria-hidden="true">
              <div className="answer-option">
                <span>A</span>
                {copy.answers[0]}
              </div>
              <div className="answer-option target-option">
                <span>B</span>
                {copy.answers[1]}
              </div>
              <div className="answer-option">
                <span>C</span>
                {copy.answers[2]}
              </div>
            </div>
          </article>

          <div
            className="listening-bubble"
            aria-hidden={activePhase !== "listening"}
          >
            <span className="mic-icon">
              <MicrophoneIcon />
            </span>
            <div className="listening-copy">
              <strong>{copy.listening}</strong>
              <span>{copy.voicePrompt}</span>
            </div>
            <span className="sound-wave" aria-hidden="true">
              <i />
              <i />
              <i />
              <i />
            </span>
          </div>

          <div
            className="thinking-pill"
            aria-hidden={activePhase !== "thinking"}
          >
            <TroMark />
            <span>{copy.thinking}</span>
            <i />
            <i />
            <i />
          </div>

          <article
            className="solution-card"
            aria-hidden={activePhase !== "solved"}
          >
            <header>
              <span className="solution-brand">
                <TroMark />
                Tro
              </span>
              <span className="solution-check">{copy.understood}</span>
            </header>
            <p className="solution-eyebrow">{copy.solutionEyebrow}</p>
            <h3>{copy.solutionTitle}</h3>
            <ol className="abc-list">
              <li className="abc-step abc-step--a">
                <span>A</span>
                <div>
                  <strong>{copy.steps[0].title}</strong>
                  <p>{copy.steps[0].body}</p>
                </div>
              </li>
              <li className="abc-step abc-step--b">
                <span>B</span>
                <div>
                  <strong>{copy.steps[1].title}</strong>
                  <p>{copy.steps[1].body}</p>
                </div>
              </li>
              <li className="abc-step abc-step--c">
                <span>C</span>
                <div>
                  <strong>{copy.steps[2].title}</strong>
                  <p>{copy.steps[2].body}</p>
                </div>
              </li>
            </ol>
            <div className="final-answer">
              <span>{copy.answer}</span>
              <strong>B</strong>
              <p>{copy.encouragement}</p>
            </div>
          </article>

          <div className="simulated-cursor" aria-hidden="true">
            <span className="cursor-ripple" />
            <PointerIcon />
            <span className="cursor-tag">Tro</span>
          </div>
        </div>
      </div>
    </section>
  );
}

function PartnersSection({ copy }: { copy: SiteCopy["partners"] }) {
  return (
    <section
      className="partners-section"
      id="partners"
      aria-labelledby="partners-title"
    >
      <div className="partners-shell">
        <div className="partners-intro">
          <div>
            <p className="section-label">{copy.label}</p>
            <h2 id="partners-title">{copy.title}</h2>
          </div>
          <p>{copy.intro}</p>
        </div>

        <div className="partners-list">
          {copy.entries.map((partner) => (
            <a
              className="partner-card"
              href={partner.website}
              target="_blank"
              rel="noreferrer"
              aria-label={partner.linkLabel}
              key={partner.website}
            >
              <span className="partner-card__logo" aria-hidden="true">
                <img
                  src={partner.logo}
                  alt=""
                  width="655"
                  height="655"
                  loading="lazy"
                />
              </span>
              <span className="partner-card__copy">
                <span className="partner-card__eyebrow">
                  <i aria-hidden="true" />
                  {partner.featuredLabel}
                </span>
                <strong>{partner.name}</strong>
                <span className="partner-card__description">
                  {partner.description}
                </span>
              </span>
              <span className="partner-card__cta">
                {partner.visit}
                <span aria-hidden="true">↗</span>
              </span>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
}

const localeStorageKey = "tro-locale";

function getInitialLocale(): Locale {
  try {
    const storedLocale = window.localStorage.getItem(localeStorageKey);
    if (storedLocale === "vi" || storedLocale === "en") {
      return storedLocale;
    }
  } catch {
    // Storage can be unavailable in strict privacy modes; Vietnamese stays the default.
  }

  return "vi";
}

export function LandingPage() {
  const [locale, setLocale] = useState<Locale>(getInitialLocale);
  const copy = siteCopy[locale];

  useEffect(() => {
    document.documentElement.lang = locale;
    document.title = copy.meta.title;
    document
      .querySelector<HTMLMetaElement>('meta[name="description"]')
      ?.setAttribute("content", copy.meta.description);

    try {
      window.localStorage.setItem(localeStorageKey, locale);
    } catch {
      // The page remains fully usable when persistence is unavailable.
    }
  }, [copy.meta.description, copy.meta.title, locale]);

  return (
    <main>
      <header className="site-header">
        <div className="header-left">
          <a className="brand" href="#top" aria-label={copy.header.homeLabel}>
            tro
          </a>
          <nav aria-label={copy.header.navigationLabel}>
            <a href="#demo">{copy.header.howItWorks}</a>
            <a href="#principles">{copy.header.whyTro}</a>
          </nav>
        </div>
        <a
          className="header-emblem"
          href="#top"
          aria-label={copy.header.backToTop}
        >
          <TroMark />
        </a>
        <div className="system-cluster" aria-label={copy.header.systemStatus}>
          <span className="system-icon">
            <WifiIcon />
          </span>
          <span className="system-icon system-icon--battery">
            <BatteryIcon />
          </span>
          <div
            className="language-switch"
            role="group"
            aria-label={copy.language.label}
          >
            <button
              type="button"
              className={locale === "vi" ? "is-active" : undefined}
              aria-pressed={locale === "vi"}
              aria-label={copy.language.vietnamese}
              onClick={() => setLocale("vi")}
            >
              VI
            </button>
            <button
              type="button"
              className={locale === "en" ? "is-active" : undefined}
              aria-pressed={locale === "en"}
              aria-label={copy.language.english}
              onClick={() => setLocale("en")}
            >
              EN
            </button>
          </div>
          <a className="header-cta" href="#demo">
            {copy.header.getTro}
            <span aria-hidden="true">↘</span>
          </a>
        </div>
      </header>

      <section className="hero" id="top">
        <div className="hero-desktop" aria-hidden="true">
          <article className="floating-window floating-window--lesson">
            <div className="floating-window__bar">
              <MacControls />
              <span>{copy.hero.practiceWindow}</span>
            </div>
            <div className="floating-window__body lesson-preview">
              <span>{copy.hero.topic}</span>
              <strong>f(x) = (x − 2)² + 1</strong>
              <i />
              <i />
              <i />
            </div>
          </article>

          <article className="floating-window floating-window--steps">
            <div className="floating-window__bar">
              <MacControls />
              <span>{copy.hero.explanationWindow}</span>
            </div>
            <div className="floating-window__body steps-preview">
              <span>
                <b>A</b> {copy.hero.previewSteps[0]}
              </span>
              <span>
                <b>B</b> {copy.hero.previewSteps[1]}
              </span>
              <span>
                <b>C</b> {copy.hero.previewSteps[2]}
              </span>
            </div>
          </article>

          <article className="floating-window floating-window--voice">
            <div className="floating-window__bar">
              <MacControls />
              <span>{copy.hero.voiceWindow}</span>
            </div>
            <div className="floating-window__body voice-preview">
              <span className="voice-preview__mic">
                <MicrophoneIcon />
              </span>
              <div>
                <strong>{copy.hero.listening}</strong>
                <span>{copy.hero.voicePrompt}</span>
              </div>
              <span className="sound-wave">
                <i />
                <i />
                <i />
                <i />
              </span>
            </div>
          </article>

          <div className="desktop-folder desktop-folder--one">
            <span />
            {copy.hero.notesFolder}
          </div>
          <div className="desktop-folder desktop-folder--two">
            <span />
            {copy.hero.progressFolder}
          </div>
          <span className="desktop-glyph desktop-glyph--one">⌁</span>
          <span className="desktop-glyph desktop-glyph--two">
            {"{ A·B·C }"}
          </span>
          <span className="desktop-glyph desktop-glyph--three">⌘ + /</span>
          <span className="hero-pointer">
            <PointerIcon />
          </span>
        </div>

        <div className="hero-copy">
          <p className="code-kicker">
            <span>const</span> {copy.hero.codeVariable} ={" "}
            <strong>“{copy.hero.codeValue}”</strong>;
          </p>
          <h1>tro.</h1>
          <p className="hero-tagline">{copy.hero.tagline}</p>
          <p className="hero-description">{copy.hero.description}</p>
          <div className="hero-actions">
            <a className="primary-cta" href="#demo">
              <span className="command-symbol" aria-hidden="true">
                ⌘
              </span>
              {copy.hero.primaryCta}
            </a>
            <a className="secondary-cta" href="#principles">
              {copy.hero.secondaryCta}
            </a>
          </div>
          <p className="shortcut-note">
            {copy.hero.shortcutPrefix} <kbd>⌘</kbd> <kbd>/</kbd>{" "}
            {copy.hero.shortcutSuffix}
          </p>
        </div>
        <aside className="hero-note" aria-label={copy.hero.noteAria}>
          <div className="hero-note__bar">
            <MacControls />
            <span>{copy.hero.noteWindow}</span>
          </div>
          <div className="hero-note__body">
            <span className="hero-note__icon">
              <TroMark />
            </span>
            <div>
              <p>{copy.hero.noteKicker}</p>
              <strong>{copy.hero.noteBody}</strong>
            </div>
          </div>
        </aside>
      </section>

      <PartnersSection copy={copy.partners} />

      <ComputerUseDemo copy={copy.demo} />

      <section
        className="principles"
        id="principles"
        aria-labelledby="principles-title"
      >
        <div className="principles-intro">
          <p className="section-label">{copy.principles.label}</p>
          <h2 id="principles-title">{copy.principles.title}</h2>
        </div>
        <div className="feature-grid">
          {copy.principles.features.map((feature) => (
            <article className="feature-card" key={feature.number}>
              <span>{feature.number}</span>
              <h3>{feature.title}</h3>
              <p>{feature.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="closing-section">
        <div>
          <p className="code-kicker code-kicker--light">
            <span>return</span> {copy.closing.codeObject}++;
          </p>
          <h2>
            {copy.closing.firstLine}
            <br />
            {copy.closing.secondLine}
          </h2>
        </div>
        <a href="#demo" className="closing-cta">
          {copy.closing.cta}
          <span aria-hidden="true">↗</span>
        </a>
      </section>

      <footer>
        <span className="footer-brand">
          <TroMark /> Tro
        </span>
        <p>{copy.footer.statement}</p>
        <span>© 2026 Tro</span>
      </footer>
    </main>
  );
}
