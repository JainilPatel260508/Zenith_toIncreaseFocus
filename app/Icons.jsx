const paths = {
  dashboard: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
  timer: 'M9 2h6 M12 8v5l3 2 M19 5l2 2 M12 5a8 8 0 1 0 0 16a8 8 0 0 0 0-16',
  rooms:
    'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M9 3a4 4 0 1 0 0 8a4 4 0 0 0 0-8 M22 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75',
  chart: 'M4 3v17h17 M8 15v-3 M13 15V8 M18 15V5',
  shield: 'M12 2l9 4v6c0 5-9 10-9 10S3 17 3 12V6z M8 12l3 3 5-6',
  award: 'M8 14l-2 8 6-3 6 3-2-8 M12 2a7 7 0 1 0 0 14a7 7 0 0 0 0-14',
  settings:
    'M12 8a4 4 0 1 0 0 8a4 4 0 0 0 0-8 M12 2v3 M12 19v3 M2 12h3 M19 12h3 M5 5l2 2 M17 17l2 2 M5 19l2-2 M17 7l2-2',
  play: 'M8 4l12 8-12 8z',
  pause: 'M8 5v14 M16 5v14',
  reset: 'M3 10a9 9 0 1 1 2 8 M3 3v7h7',
  arrow: 'M4 12h16 M14 6l6 6-6 6',
  plus: 'M12 5v14 M5 12h14',
  check: 'M5 12l4 4L19 6',
  close: 'M6 6l12 12 M18 6L6 18',
  flame: 'M13 2c2 6-3 6-2 10c-3-1-3-4-3-4s-5 5-3 10c3 7 14 5 14-3c0-6-6-13-6-13',
  bolt: 'M13 2L4 14h7l-1 8 10-13h-7z',
  sun: 'M12 7a5 5 0 1 0 0 10a5 5 0 0 0 0-10 M12 1v2 M12 21v2 M1 12h2 M21 12h2 M4 4l2 2 M18 18l2 2 M4 20l2-2 M18 6l2-2',
  moon: 'M21 13A9 9 0 0 1 11 3a9 9 0 1 0 10 10',
  globe: 'M12 2a10 10 0 1 0 0 20a10 10 0 0 0 0-20 M2 12h20 M12 2c-6 5-6 15 0 20c6-5 6-15 0-20',
  logout: 'M9 4H4v16h5 M9 12h13 M17 7l5 5-5 5',
  lock: 'M6 10h12v11H6z M8 10V6a4 4 0 0 1 8 0v4',
  leaf: 'M20 3C3 2 2 12 7 17c5 5 15 1 13-14z M5 21L16 9',
}
export default function Icon({ name, size = 18, ...props }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      <path d={paths[name] || paths.timer} />
    </svg>
  )
}
