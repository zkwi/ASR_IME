<script lang="ts">
  type Props = {
    title: string;
    hint: string;
  };

  let { title, hint }: Props = $props();
</script>

<main class="toast-root">
  <section class="toast-card">
    <div class="toast-icon">
      <img src="/app-icon.svg" alt="" aria-hidden="true" draggable="false" />
    </div>
    <div class="toast-copy">
      <strong>{title}</strong>
      <span>{hint}</span>
    </div>
  </section>
</main>

<style>
  .toast-root {
    display: grid;
    width: 100vw;
    height: 100vh;
    place-items: center;
    min-width: 0;
    padding: 6px;
    overflow: hidden;
    background: transparent;
  }

  .toast-card {
    display: grid;
    grid-template-columns: 34px minmax(0, 1fr);
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 100%;
    min-width: 0;
    padding: 9px 12px;
    overflow: hidden;
    color: #17222e;
    background: rgba(255, 255, 255, 0.98);
    border: 1px solid rgba(47, 140, 255, 0.18);
    border-radius: 12px;
    box-shadow: 0 8px 20px rgba(34, 61, 88, 0.16);
  }

  .toast-icon {
    display: grid;
    width: 34px;
    height: 34px;
    place-items: center;
    overflow: hidden;
    background: transparent;
    border-radius: 9px;
  }

  .toast-icon img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .toast-copy {
    display: grid;
    gap: 3px;
    min-width: 0;
    overflow: hidden;
  }

  .toast-card strong {
    display: block;
    min-width: 0;
    overflow: hidden;
    color: #152231;
    font-size: 0.88rem;
    font-weight: 700;
    line-height: 1.18;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toast-card span {
    display: block;
    min-width: 0;
    overflow: hidden;
    color: #66788a;
    font-size: 0.72rem;
    line-height: 1.24;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (prefers-reduced-motion: no-preference) {
    .toast-card {
      animation: toast-enter 180ms ease-out;
    }

    @keyframes toast-enter {
      from {
        opacity: 0;
        transform: translateY(6px) scale(0.98);
      }

      to {
        opacity: 1;
        transform: translateY(0) scale(1);
      }
    }
  }
</style>
