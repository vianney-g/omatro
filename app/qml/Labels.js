.pragma library

// Displayed text: ranks, suits, effects, numbers (hand names come from the engine).

var SUITS = { hearts: "♥", diamonds: "♦", clubs: "♣", spades: "♠" }

function rank(r) { return r <= 10 ? String(r) : ["J", "Q", "K", "A"][r - 11] }

function suit(s) { return SUITS[s] || "" }

function isRed(s) { return s === "hearts" || s === "diamonds" }

// English number format: 12,345 and 2.5, then 1.234e11 past a hundred billion.
// Infinite targets come from the engine as null.
function number(n) {
  if (n === null || !isFinite(n)) return "\u221e"
  n = Math.floor(n)
  if (n >= 1e11) {
    var e = Math.floor(Math.log10(n))
    return (n / Math.pow(10, e)).toFixed(3) + "e" + e
  }
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ",")
}

function money(n) { return "$" + n }

// "1 card", "2 cards", "1–3 cards".
function range(min, max) {
  return (min === max ? String(min) : min + "–" + max) + (max === 1 ? " card" : " cards")
}

function mult(m) {
  return Number.isInteger(m) ? String(m) : m.toFixed(1)
}

// An engine effect: {"chips": 11}, {"mult": 2}, {"x_mult": 2}, {"money": 2} or "again".
function effect(e) {
  if (e === "again") return "Again!"
  if (e.money !== undefined) return "+" + money(e.money)
  if (e.chips !== undefined) return "+" + e.chips
  if (e.mult !== undefined) return "+" + mult(e.mult) + " mult"
  if (e.x_mult !== undefined) return "×" + mult(e.x_mult) + " mult"
  return ""
}

function isChips(e) { return e.chips !== undefined }

function isMoney(e) { return e.money !== undefined }

var BLIND_STATUSES = { upcoming: "Upcoming", select: "Next", current: "In progress", defeated: "Defeated", skipped: "Skipped" }

function blindStatus(s) { return BLIND_STATUSES[s] || "" }
