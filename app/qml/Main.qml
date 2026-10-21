import QtCore
import QtQuick
import QtQuick.Layouts
import omatro
import "Labels.js" as Labels

// Game window. The rules live in the Rust engine, called directly (Engine
// object) with the engine's JSON protocol. This file only displays and
// animates.
Window {
  id: root

  title: "omatro"
  visible: true
  color: Colors.background
  // Sizes from the theme scale alone: `Style.space` depends on the window size.
  width: 1100 * Style.themeScale
  height: 800 * Style.themeScale
  minimumWidth: designWidth / 2
  minimumHeight: designHeight / 2

  // Smallest window the layout fits in at full size; below it, everything
  // shrinks together (a tiled window can be smaller than the minimum size).
  // Width: margins, left column and the 8 cards of the hand (1032).
  readonly property real designWidth: 1040 * Style.themeScale
  // Height: the shop, with the joker row (230 high), the offers (230) and,
  // under them, the pull requests and packs (150).
  readonly property real designHeight: 920 * Style.themeScale
  // Width of the play area (jokers, table, hand, shop), from the window's:
  // never from the play area's own width, which follows its content and would
  // make a layout loop (the shop collapsed after a purchase).
  readonly property real playWidth: width - 3 * Style.spacing.huge - Style.space(250)
  Binding {
    target: Style
    property: "fit"
    value: Math.min(1, root.width / root.designWidth, root.height / root.designHeight)
  }

  // ---- engine and theme --------------------------------------------------
  Engine { id: engine }

  function send(command) {
    receive(engine.handle(JSON.stringify(command)))
  }

  function receive(line) {
    var r
    try { r = JSON.parse(line) } catch (e) { notice = I18n.tr("Unreadable response from the engine."); return }
    // What the collection gained: shown once a hand's score is told.
    if (r.news) {
      if (r.event === "scored") pendingNews = pendingNews.concat(r.news)
      else showNews(r.news)
    }
    if (r.event === "state") applyState(r.state)
    else if (r.event === "collection") collection = r.collection
    else if (r.event === "preview") showPreview(r)
    else if (r.event === "scored") startScoring(r)
    else if (r.event === "error") notice = r.message
    else if (r.event === "glossary") Hints.terms = r.terms
    else if (r.event === "language") {
      Labels.setLanguage(r.lang, r.table)
      I18n.lang = r.lang
      I18n.languages = r.languages
      I18n.table = r.table
    }
    else if (r.event === "saved_game") {
      savedGame = r.state
      saveProblem = r.problem ? I18n.tr("Couldn't resume your game: {why}.", { why: r.problem }) : ""
    }
    else if (r.event === "decks") {
      decks = r.decks
      stakes = r.stakes
      newGameScreen.stake = Math.max(0, stakes.findIndex(function(s) { return s.id === settings.lastStake }))
      var last = decks.findIndex(function(d) { return d.id === settings.lastDeck })
      newGameScreen.current = Math.max(0, last)
    }
  }

  // Omarchy theme switch: reload its files if they changed.
  Timer {
    interval: 1000
    running: true
    repeat: true
    onTriggered: OmarchyTheme.refresh()
  }

  // A game left in progress is offered first; otherwise, the deck choice.
  Component.onCompleted: {
    var lang = settings.language || (Qt.locale().name.indexOf("fr") === 0 ? "fr" : "en")
    send({ cmd: "language", lang: lang })
    send({ cmd: "glossary" })
    send({ cmd: "saved_game" })
    if (savedGame !== null) resuming = true
    else newGame()
    keyCatcher.forceActiveFocus()
  }

  // ---- game state --------------------------------------------------------
  property var game: null        // last state sent by the engine
  property var hand: []          // cards shown in the hand
  property var selected: []      // hand indices, in click order
  property var preview: null
  property var scoring: null     // score breakdown being animated
  property var pendingState: null
  property int stepIndex: -1
  property int chipSteps: 0      // chips steps sounded, a note higher each
  property bool showTotal: false
  property real liveChips: 0
  property real liveMult: 0
  property real shownScore: 0
  // Money as shown: it counts up as it is earned (scoring steps, cash-out)
  // and drops at once when spent; each change floats beside it.
  property real shownMoney: 0
  property bool moneyCounts: true
  Behavior on shownMoney {
    enabled: root.moneyCounts
    NumberAnimation { duration: 500; easing.type: Easing.OutCubic }
  }
  function showMoney(money) {
    var delta = money - Math.round(shownMoney)
    if (delta === 0) return
    moneyCounts = delta > 0
    shownMoney = money
    moneyCounts = true
    moneyDelta.show(delta)
  }
  property string notice: ""
  property bool infoOpen: false   // fastfetch panel

  // ---- collection and achievements (C), and what they gain (news)
  property var collection: null
  property bool collectionOpen: false
  property var pendingNews: []
  property var news: []           // shown at the top right, oldest first
  function openCollection() {
    send({ cmd: "collection" })
    collectionOpen = true
  }
  function showNews(items) {
    // Many finds at once make a single line.
    var found = items.filter(function(n) { return n.kind === "found" })
    var shown = items.filter(function(n) { return n.kind !== "found" })
    if (found.length > 3)
      shown.unshift({ kind: "found", name: I18n.tr("{n} new things", { n: found.length }), text: "" })
    else
      shown = found.concat(shown)
    if (items.some(function(n) { return n.kind === "achievement" || n.kind === "unlocked" })) sound("achievement")
    else if (items.some(function(n) { return n.kind === "found" })) sound("found")
    news = news.concat(shown).slice(-6)
    newsTimer.restart()
  }
  Timer {
    id: newsTimer
    interval: 3500
    repeat: true
    onTriggered: {
      root.news = root.news.slice(1)
      if (root.news.length === 0) stop()
    }
  }
  Behavior on shownScore { NumberAnimation { duration: 900; easing.type: Easing.OutCubic } }
  // Flames on the blind score while a single hand beats the blind: 1 at the
  // target, 2 at 1.5×, 3 at 3×; they die down a moment after the hand.
  property int fireHeat: 0
  function heatFor(total) {
    var target = game ? game.target : null
    if (target === null || target <= 0) return 0
    return total >= 3 * target ? 3 : total >= 1.5 * target ? 2 : total >= target ? 1 : 0
  }
  Timer { id: fireOut; interval: 1800; onTriggered: root.fireHeat = 0 }

  readonly property bool busy: scoring !== null
  readonly property bool playing: game !== null && game.phase === "playing" && !busy
  readonly property bool shopping: game !== null && game.phase === "shop" && !busy
  readonly property bool choosing: game !== null && game.phase === "blind_select"
  // Jokers can be sold or moved between hands, in the shop and before a blind.
  readonly property bool jokerControls: game !== null
    && (game.phase === "playing" || game.phase === "shop" || game.phase === "blind_select")
  readonly property bool jokersEditable: jokerControls && !busy
  // A pack is open; coreutils packs draw a hand to select cards from.
  readonly property bool inPack: game !== null && game.phase === "pack"
  readonly property bool packHand: inPack && game.opened !== null && game.opened.with_hand
  // Cards of the hand can be selected: in a blind, or in a coreutils pack.
  readonly property bool selecting: playing || (packHand && !busy)
  readonly property var currentBlind: {
    if (!game) return null
    for (var i = 0; i < game.blinds.length; i++)
      if (game.blinds[i].kind === game.blind) return game.blinds[i]
    return null
  }
  readonly property bool endless: game !== null && game.ante > game.winning_ante
  // The boss at work: cards it hides or disables, by card id (the hand shown
  // during the score animation is not the state's hand).
  readonly property string bossName: game !== null && game.boss_active ? game.boss.name : ""
  // The ante's boss is still ahead, or being played.
  readonly property bool upcomingBoss: game !== null && game.phase !== "lost"
    && game.blinds.some(function(b) { return b.kind === "boss" && b.status !== "defeated" })
  readonly property var faceDownIds: game ? game.hand.filter(function(c, i) { return game.face_down[i] })
    .map(function(c) { return c.id }) : []
  readonly property var debuffedIds: game ? game.hand.filter(function(c, i) { return game.debuffed[i] })
    .map(function(c) { return c.id }) : []
  readonly property int forcedId: game && game.forced !== null ? game.hand[game.forced].id : -1
  readonly property int handLevel: scoring ? scoring.hand_level
    : (preview && preview.hand ? preview.hand_level : 0)
  readonly property string handName: scoring ? scoring.hand_name
    : (preview && preview.hand ? preview.hand_name : "")

  // ---- new game: pick the deck first ------------------------------------
  property var decks: []
  property var stakes: []
  property bool choosingDeck: false
  Settings {
    id: settings
    category: "omatro"
    property string lastDeck: ""
    property string lastStake: ""
    // The language chosen in the game ("": the system's).
    property string language: ""
    property bool music: true
    property real musicVolume: 0.5
    property bool sounds: true
    property real soundVolume: 0.5
  }

  // ---- music (M switches it on and off); none in scripted runs, unless
  // they ask (the "audio" step)
  property bool scriptAudio: false
  readonly property bool audible: engine.script() === "" || scriptAudio
  function toggleMusic() { settings.music = !settings.music }
  function setMusicVolume(v) {
    settings.musicVolume = Math.round(Math.max(0, Math.min(1, v)) * 10) / 10
    if (!settings.music) settings.music = true
  }

  // ---- sound effects (Shift+M), by name (Sounds.qml)
  function toggleSounds() { settings.sounds = !settings.sounds }
  function setSoundVolume(v) {
    settings.soundVolume = Math.round(Math.max(0, Math.min(1, v)) * 10) / 10
    if (!settings.sounds) settings.sounds = true
  }
  function sound(name) {
    if (soundsPlayer.item) soundsPlayer.item.play(name)
  }
  // Sends a command and sounds how it went: its sound once the state
  // changed, the error one if refused.
  function chipSound() {
    if (soundsPlayer.item) soundsPlayer.item.chips(chipSteps++)
  }
  function act(command, name) {
    var before = game
    send(command)
    sound(game !== before ? name : "error")
  }
  Loader {
    id: soundsPlayer
    source: "Sounds.qml"
  }
  Binding { target: soundsPlayer.item; when: soundsPlayer.item !== null; property: "active"; value: settings.sounds && root.audible }
  Binding { target: soundsPlayer.item; when: soundsPlayer.item !== null; property: "volume"; value: settings.soundVolume }
  Loader {
    id: musicPlayer
    source: "Music.qml"
  }
  Binding { target: musicPlayer.item; when: musicPlayer.item !== null; property: "calm"; value: root.game !== null && !root.choosingDeck && !root.resuming
    && (root.game.phase === "playing" || root.game.phase === "blind_select") }
  Binding { target: musicPlayer.item; when: musicPlayer.item !== null; property: "playing"; value: settings.music && root.audible }
  Binding { target: musicPlayer.item; when: musicPlayer.item !== null; property: "volume"; value: settings.musicVolume }
  Connections {
    target: musicPlayer.item
    function onStarted(title) { root.showNews([{ kind: "music", name: title, text: "" }]) }
  }

  // ---- language: the engine translates its text and sends the catalog for
  // the interface's; what it already sent comes again, translated
  function setLanguage(lang) {
    settings.language = lang
    send({ cmd: "language", lang: lang })
    send({ cmd: "glossary" })
    send({ cmd: "decks" })
    if (resuming) send({ cmd: "saved_game" })
    if (collectionOpen) send({ cmd: "collection" })
    if (game !== null && !busy) send({ cmd: "state" })
  }
  // L: the next language.
  function nextLanguage() {
    var ids = I18n.languages.map(function(l) { return l.id })
    if (ids.length > 0) setLanguage(ids[(ids.indexOf(I18n.lang) + 1) % ids.length])
  }

  // ---- resuming the game left in progress --------------------------------
  property var savedGame: null
  property bool resuming: false
  // Why the saved game could not be resumed, shown on the deck choice.
  property string saveProblem: ""
  function resumeGame() {
    resuming = false
    send({ cmd: "resume" })
    if (game === null) newGame()
  }
  function declineResume() {
    resuming = false
    newGame()
  }

  function newGame() {
    // Again each time: a deck may have been unlocked since.
    send({ cmd: "decks" })
    if (Hints.terms.length === 0) send({ cmd: "glossary" })
    var last = decks.findIndex(function(d) { return d.id === settings.lastDeck })
    newGameScreen.current = Math.max(0, last)
    choosingDeck = true
  }
  function startGame(deck, stake, seed) {
    settings.lastDeck = deck
    settings.lastStake = stake
    choosingDeck = false
    notice = ""
    var command = { cmd: "new_game", deck: deck, stake: stake }
    if (seed !== "") command.seed_code = seed
    send(command)
    // Typed again for the next game, so that none is seeded by mistake.
    newGameScreen.seed = ""
    keyCatcher.forceActiveFocus()
  }

  function applyState(s) {
    if (game !== null && game.seed_code === s.seed_code) soundTransition(game, s)
    // Another game (new or resumed): its money shows as it is.
    if (game === null || game.seed_code !== s.seed_code) {
      moneyCounts = false
      shownMoney = s.money
      moneyCounts = true
    }
    game = s
    hand = s.hand
    // The Clippy's card stays selected.
    selected = s.forced !== null ? [s.forced] : []
    preview = null
    liveChips = 0
    liveMult = 0
    shownScore = s.score
    showMoney(s.money)
  }

  // What a phase change sounds like.
  function soundTransition(from, to) {
    if (from.phase === to.phase) return
    if (to.phase === "playing" && from.phase === "blind_select") sound(to.blind === "boss" ? "boss" : "deal")
    else if (to.phase === "won") sound(to.victory ? "victory" : "blind-won")
    else if (to.phase === "lost") sound("game-over")
    else if (to.phase === "shop" && from.phase === "won") sound("cash-out")
    else if (to.phase === "pack") sound("pack")
  }

  function toggleCard(i) {
    if (!selecting || i < 0 || i >= hand.length) return
    if (playing && game.forced === i) { notice = I18n.tr("The Clippy insists on this card."); return }
    var s = selected.slice()
    var k = s.indexOf(i)
    if (k >= 0) s.splice(k, 1)
    else if (s.length < 5) s.push(i)
    else return
    selected = s
    sound(k >= 0 ? "deselect" : "select")
    notice = ""
    if (playing) send({ cmd: "preview", cards: s })
  }

  // Moves or sorts the hand; the selection follows its cards.
  function rearrange(command) {
    if (selecting) keepingSelection(command)
  }
  // Sends a command that leaves the hand's cards as they were (or moves
  // them): the selection, and its preview, stay.
  function keepingSelection(command) {
    var ids = selected.map(function(i) { return hand[i].id })
    send(command)
    selected = ids.map(function(id) { return hand.findIndex(function(c) { return c.id === id }) })
      .filter(function(i) { return i >= 0 })
    if (playing && selected.length > 0) send({ cmd: "preview", cards: selected })
  }
  function moveCard(from, to) {
    to = Math.max(0, Math.min(hand.length - 1, to))
    if (from !== to) rearrange({ cmd: "move_card", from: from, to: to })
  }
  function sortHand(by) {
    rearrange({ cmd: "sort_hand", by: by })
  }

  function showPreview(r) {
    if (JSON.stringify(r.cards) !== JSON.stringify(selected)) return
    preview = r
    liveChips = r.base_chips
    liveMult = r.base_mult
  }

  function play() {
    if (!playing || selected.length === 0) return
    sound("play")
    send({ cmd: "play", cards: selected })
  }

  function discard() {
    if (!playing || selected.length === 0) return
    if (game.discards_left === 0) { notice = I18n.tr("No discards left."); sound("error"); return }
    act({ cmd: "discard", cards: selected }, "discard")
  }

  // Enter: play the blind or the hand, cash out, leave the shop or start over.
  function confirm() {
    if (busy || !game) return
    notice = ""
    if (game.phase === "blind_select") send({ cmd: "select_blind" })
    else if (game.phase === "won") send({ cmd: "cash_out" })
    else if (game.phase === "shop") send({ cmd: "next_blind" })
    else if (game.phase === "lost") newGame()
    else play()
  }

  function skipBlind() {
    if (busy || !choosing) return
    notice = ""
    act({ cmd: "skip_blind" }, "skip")
  }

  // Shortcuts for testing, refused by the engine unless OMATRO_DEBUG=1.
  function cheat(c) {
    if (busy || !game) return
    c.cmd = "debug"
    send(c)
  }

  // ---- shop --------------------------------------------------------------
  // Money, plus what Cloud Credits lets you owe.
  function canPay(price) {
    return game.money + game.credit >= price
  }

  function canBuy(offer) {
    if (offer.kind === "card") return shopping && canPay(offer.price)
    if (offer.kind !== "joker")
      return shopping && canPay(offer.price) && game.consumables.length < game.consumable_slots
    var transparent = offer.edition !== null && offer.edition.id === "transparent"
    return shopping && canPay(offer.price) && (transparent || game.jokers.length < game.max_jokers)
  }
  // Themes and commands that need no cards in hand can be used at once,
  // even with a full clipboard.
  function canBuyAndUse(offer) {
    return shopping && (offer.kind === "theme" || offer.kind === "command")
      && !offer.needs_hand && canPay(offer.price)
  }
  function buyAndUse(i) {
    if (!shopping || i < 0 || i >= game.shop.length) return
    var offer = game.shop[i]
    var before = game.shop.length
    act({ cmd: "buy_and_use", offer: i }, "use")
    if (game.shop.length < before)
      notice = I18n.tr(offer.kind === "theme" ? "Applied {name}." : "Ran {name}.", { name: offer.name })
  }
  function buy(i) {
    if (shopping && i >= 0 && i < game.shop.length) act({ cmd: "buy", offer: i }, "buy")
  }
  function sell(i) {
    if (jokersEditable) {
      keepingSelection({ cmd: "sell", joker: i })
      sound("sell")
    }
  }
  // Whether a consumable can be used now: commands that change cards need
  // the right number of them selected, during a blind.
  function canUse(c) {
    if (busy || !game || game.phase === "lost") return false
    if (c.needs_hand && !selecting) return false
    if (c.max_cards === 0) return true
    return selected.length >= c.min_cards && selected.length <= c.max_cards
  }
  // ---- packs -------------------------------------------------------------
  function canMerge(i) {
    return shopping && i < game.vouchers_for_sale.length && canPay(game.vouchers_for_sale[i].price)
  }
  function buyVoucher(i) {
    if (canMerge(i)) {
      var name = game.vouchers_for_sale[i].name
      act({ cmd: "buy_voucher", index: i }, "merge")
      notice = I18n.tr("Merged {name}.", { name: name })
    }
  }
  function rerollBoss() {
    if (!busy && game && game.boss_reroll_cost !== null) send({ cmd: "reroll_boss" })
  }
  function buyPack(i) {
    if (shopping && i >= 0 && i < game.packs.length) send({ cmd: "buy_pack", pack: i })
  }
  function canBuyPack(pack) {
    return shopping && canPay(pack.price)
  }
  // A pack's choice: jokers need a slot, commands that change cards a selection.
  function canPick(choice) {
    if (busy || !inPack) return false
    if (choice.kind === "joker") {
      var transparent = choice.edition !== null && choice.edition.id === "transparent"
      return transparent || game.jokers.length < game.max_jokers
    }
    if (choice.max_cards > 0)
      return selected.length >= choice.min_cards && selected.length <= choice.max_cards
    return true
  }
  function pickChoice(i) {
    if (!inPack || i < 0 || i >= game.opened.choices.length) return
    var choice = game.opened.choices[i]
    if (!canPick(choice)) {
      notice = choice.max_cards > 0
        ? I18n.tr("{name}: select {range}.", { name: choice.name, range: Labels.range(choice.min_cards, choice.max_cards) })
        : I18n.tr("No room for another joker.")
      sound("error")
      return
    }
    notice = ""
    act({ cmd: "pick", choice: i, cards: choice.max_cards > 0 ? selected : [] }, "pick")
  }
  // Digit keys pick the item with that number: cards of the hand; in the
  // shop, offers, then packs, then pull requests; in a pack, its choices.
  // With Shift, the other action where there is one: Use now for a shop
  // offer, a choice of a pack drawn with a hand (whose digits select cards).
  function pressDigit(i, shifted) {
    if (shopping) {
      var offers = game.shop.length, packs = game.packs.length
      if (i < offers) {
        if (shifted && canBuyAndUse(game.shop[i])) buyAndUse(i)
        else buy(i)
      }
      else if (i < offers + packs) buyPack(i - offers)
      else buyVoucher(i - offers - packs)
    }
    else if (inPack && (!packHand || shifted)) pickChoice(i)
    else toggleCard(i)
  }
  // The key of item i in pressDigit's order: 1-9, then 0.
  function keyName(i) {
    return i < 9 ? String(i + 1) : i === 9 ? "0" : ""
  }

  function skipPack() {
    if (inPack && !busy) send({ cmd: "skip_pack" })
  }

  function useConsumable(i) {
    if (!game || i >= game.consumables.length) return
    var c = game.consumables[i]
    if (!canUse(c)) {
      if (c.max_cards > 0) notice = I18n.tr("{name}: select {range} during a blind.", { name: c.name, range: Labels.range(c.min_cards, c.max_cards) })
      else if (c.needs_hand) notice = I18n.tr("{name}: use it during a blind, on the cards in hand.", { name: c.name })
      sound("error")
      return
    }
    act({ cmd: "use_consumable", index: i, cards: c.max_cards > 0 ? selected : [] }, "use")
    notice = I18n.tr(c.kind === "theme" ? "Applied {name}." : "Ran {name}.", { name: c.name })
  }
  function sellConsumable(i) {
    if (!busy) act({ cmd: "sell_consumable", index: i }, "sell")
  }
  // Moves a joker (Fork copies its right neighbour); a pinned joker stays
  // pinned.
  function moveJoker(from, to) {
    if (!jokersEditable) return
    to = Math.max(0, Math.min(game.jokers.length - 1, to))
    if (from === to) return
    keepingSelection({ cmd: "move_joker", from: from, to: to })
    if (jokerRow.pinned === from) jokerRow.pinned = to
  }
  function reroll() {
    if (!shopping) return
    if (!canPay(game.reroll_cost)) { notice = I18n.tr("Not enough money to reroll."); sound("error"); return }
    act({ cmd: "reroll" }, "reroll")
  }

  // ---- score animation ---------------------------------------------------
  // The played cards move to the table, then each breakdown step (card or
  // joker) bounces and updates chips × mult.
  function startScoring(r) {
    var rest = []
    for (var i = 0; i < hand.length; i++)
      if (selected.indexOf(i) < 0) rest.push(hand[i])
    hand = rest
    selected = []
    preview = null
    scoring = r.score
    pendingState = r.state
    stepIndex = -1
    showTotal = false
    liveChips = r.score.base_chips
    liveMult = r.score.base_mult
    fireOut.stop()
    fireHeat = 0
    chipSteps = 0
    stepTimer.interval = 600
    stepTimer.start()
  }

  function nextStep() {
    stepTimer.interval = 420
    var steps = scoring.steps
    stepIndex++
    if (stepIndex < steps.length) {
      var step = steps[stepIndex]
      liveChips = step.chips
      liveMult = step.mult
      if (Labels.isMoney(step.effect)) showMoney(Math.round(shownMoney) + step.effect.money)
      // The counters react: chips and mult pulse with sparks, ×mult shakes.
      var e = step.effect
      if (Labels.isChips(e)) { chipsCounter.pulse(); sparks.burst(chipsCounter, "chips"); chipSound() }
      else if (e.mult !== undefined) { multCounter.pulse(); sparks.burst(multCounter, "mult"); sound("mult") }
      else if (e.x_mult !== undefined) { multCounter.pulse(); multCounter.shake(); sparks.burst(multCounter, "warm"); sound("x-mult") }
      else if (Labels.isMoney(e)) sound("money")
      else if (e === "debuffed") sound("debuffed")
      else if (e === "balance") { chipsCounter.pulse(); multCounter.pulse() }
      fireHeat = Math.max(fireHeat, heatFor(step.chips * step.mult))
      var color = step.effect === "debuffed" ? Colors.muted
        : step.effect === "again" ? Colors.foreground
        : Labels.isChips(step.effect) || Labels.isMoney(step.effect) ? Colors.accent : Colors.urgent
      var view = step.source.kind === "card" ? tableRepeater.itemAt(step.source.index)
        : step.source.kind === "held" ? handRepeater.itemAt(step.source.index)
        : step.source.kind === "joker" ? jokerRepeater.itemAt(step.source.index)
        : null
      if (step.effect === "balance") notice = I18n.tr("Hyprland Dwindle balanced chips and mult")
      if (view) view.flash(Labels.effect(step.effect), color)
    } else if (stepIndex === steps.length) {
      showTotal = true
      // The blind score counts up with the total, not after it.
      shownScore = pendingState.score
      if (heatFor(scoring.total) > 0) sound("fire")
      fireHeat = Math.max(fireHeat, heatFor(scoring.total))
      stepTimer.interval = 900
    } else {
      stepTimer.stop()
      notice = I18n.tr("{hand}: {points} points", { hand: scoring.hand_name, points: Labels.number(scoring.total) })
        + (scoring.notes.length ? " · " + scoring.notes.join(" ") : "")
      var s = pendingState
      scoring = null
      pendingState = null
      showTotal = false
      applyState(s)
      if (s.phase === "playing") sound("draw")
      if (pendingNews.length) {
        showNews(pendingNews)
        pendingNews = []
      }
      fireOut.restart()
    }
  }

  Timer {
    id: stepTimer
    repeat: true
    onTriggered: root.nextStep()
  }

  // ---- window ------------------------------------------------------------
  component Box: Rectangle {
    radius: Style.cornerRadius
    color: Style.normalFill
    border.width: Style.normalBorderWidth
    border.color: Style.normalBorderColor
  }

  // A zone of the shop: a frame with its title, its items side by side.
  component ShopZone: Rectangle {
    property string title
    default property alias items: zoneRow.data
    width: zoneRow.implicitWidth + 2 * Style.spacing.xxl
    height: zoneTitle.implicitHeight + zoneRow.implicitHeight + 2 * Style.spacing.xxl + Style.spacing.md
    radius: Style.cornerRadius
    color: Style.normalFill
    border.width: Style.normalBorderWidth
    border.color: Style.normalBorderColor
    Caption {
      id: zoneTitle
      anchors { left: parent.left; top: parent.top; margins: Style.spacing.xxl }
      text: parent.title
    }
    Row {
      id: zoneRow
      anchors { left: parent.left; top: zoneTitle.bottom; leftMargin: Style.spacing.xxl; topMargin: Style.spacing.md }
      spacing: Style.space(16)
    }
  }

  // Chips or mult while scoring: the number pulses when it grows, the box
  // shakes on a ×mult.
  component Counter: Rectangle {
    id: counter
    property color tint
    property string value
    width: Style.space(90); height: Style.space(44)
    radius: Style.cornerRadius
    color: Util.alpha(tint, 0.18)
    border.width: Style.normalBorderWidth
    border.color: tint
    transform: Translate { id: shift }
    Text {
      id: counterText
      anchors.centerIn: parent
      text: counter.value
      color: counter.tint
      font.family: Style.font.family
      font.pixelSize: Style.font.heading
      font.bold: true
    }
    function pulse() { pulseAnimation.restart() }
    function shake() { shakeAnimation.restart() }
    SequentialAnimation {
      id: pulseAnimation
      NumberAnimation { target: counterText; property: "scale"; to: 1.35; duration: 80; easing.type: Easing.OutQuad }
      NumberAnimation { target: counterText; property: "scale"; to: 1; duration: 260; easing.type: Easing.OutBack }
    }
    SequentialAnimation {
      id: shakeAnimation
      NumberAnimation { target: shift; property: "x"; to: -Style.space(7); duration: 40 }
      NumberAnimation { target: shift; property: "x"; to: Style.space(7); duration: 60 }
      NumberAnimation { target: shift; property: "x"; to: -Style.space(4); duration: 50 }
      NumberAnimation { target: shift; property: "x"; to: 0; duration: 60 }
    }
  }

  component Caption: Text {
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
  }

  component Value: Text {
    color: Colors.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.display
    font.bold: true
  }

  Item {
    id: keyCatcher
    anchors.fill: parent
    focus: true

    Keys.onPressed: function(event) { pressKey(event) }

    // The digit row by place (0 for key 1, 9 for key 0), whatever the layout
    // types there (AZERTY and BÉPO give symbols without Shift); the keypad
    // and scripted keys by their key. -1 for any other key.
    function digitOf(event) {
      var code = event.nativeScanCode
      if (code >= 10 && code <= 19) return code - 10
      var k = event.key
      return k >= Qt.Key_1 && k <= Qt.Key_9 ? k - Qt.Key_1 : k === Qt.Key_0 ? 9 : -1
    }

    function pressKey(event) {
      var k = event.key
      var digit = digitOf(event)
      if (k === Qt.Key_Q && (event.modifiers & Qt.ControlModifier)) Qt.quit()
      else if (k === Qt.Key_L) root.nextLanguage()
      else if (k === Qt.Key_M && (event.modifiers & Qt.ShiftModifier)) root.toggleSounds()
      else if (k === Qt.Key_M) root.toggleMusic()
      else if (root.resuming) {
        if (k === Qt.Key_Return || k === Qt.Key_Enter) root.resumeGame()
        else if (k === Qt.Key_N) root.declineResume()
        else return
      }
      else if (root.collectionOpen) {
        if (k === Qt.Key_Escape || k === Qt.Key_C) root.collectionOpen = false
        else if (k === Qt.Key_Left) collectionScreen.move(-1)
        else if (k === Qt.Key_Right) collectionScreen.move(1)
        else if (k === Qt.Key_Up) collectionScreen.scroll(-1)
        else if (k === Qt.Key_Down) collectionScreen.scroll(1)
        else return
      }
      else if (k === Qt.Key_C && !(event.modifiers & Qt.ControlModifier)) root.openCollection()
      else if (root.choosingDeck) {
        if ((k === Qt.Key_Left || k === Qt.Key_Right) && (event.modifiers & Qt.ShiftModifier))
          newGameScreen.moveStake(k === Qt.Key_Left ? -1 : 1)
        else if (k === Qt.Key_Left) newGameScreen.move(-1)
        else if (k === Qt.Key_Right) newGameScreen.move(1)
        else if (k === Qt.Key_Up) newGameScreen.move(-3)
        else if (k === Qt.Key_Down) newGameScreen.move(3)
        else if (k === Qt.Key_Return || k === Qt.Key_Enter) newGameScreen.startChosen()
        else if (k === Qt.Key_S) newGameScreen.editSeed()
        else if (k === Qt.Key_Escape && root.game !== null) root.choosingDeck = false
        else return
      }
      else if (k === Qt.Key_I) root.infoOpen = !root.infoOpen
      else if (root.infoOpen) {
        if (k === Qt.Key_Escape) root.infoOpen = false
        else if (k === Qt.Key_C && (event.modifiers & Qt.ControlModifier)) fastFetch.copySeed()
        else return
      }
      else if (k === Qt.Key_C && (event.modifiers & Qt.ControlModifier) && root.game !== null && root.game.phase === "lost")
        endSeed.copy()
      else if (digit >= 0 && (event.modifiers & Qt.ControlModifier))
        root.useConsumable(digit)
      else if ((k === Qt.Key_C && (event.modifiers & Qt.ControlModifier)) || (k === Qt.Key_Escape && root.inPack))
        root.skipPack()
      else if (digit >= 0)
        root.pressDigit(digit, (event.modifiers & Qt.ShiftModifier) !== 0)
      else if ((k === Qt.Key_Left || k === Qt.Key_Right) && (event.modifiers & Qt.ShiftModifier)) {
        var step = k === Qt.Key_Left ? -1 : 1
        if (root.selected.length === 1) root.moveCard(root.selected[0], root.selected[0] + step)
        else if (jokerRow.pinned >= 0) root.moveJoker(jokerRow.pinned, jokerRow.pinned + step)
      }
      else if (k === Qt.Key_Tab || k === Qt.Key_Backtab)
        root.sortHand(root.game && root.game.sort_by === "rank" ? "suit" : "rank")
      else if (k === Qt.Key_J) jokerRow.cycle()
      else if (k === Qt.Key_R) root.reroll()
      else if (k === Qt.Key_S) root.skipBlind()
      else if (k === Qt.Key_B) root.rerollBoss()
      else if (k === Qt.Key_F1) root.cheat({ stickers: ["welded", "worn", "leased"] })
      else if (k === Qt.Key_F5) root.cheat({ win: true })
      else if (k === Qt.Key_F6) root.cheat({ money: root.game.money + 100 })
      else if (k === Qt.Key_F7) root.cheat({ ante: root.game.winning_ante, blind: "boss" })
      else if (k === Qt.Key_F2) root.cheat({ random_oligarchy: true })
      else if (k === Qt.Key_F4) root.cheat({ next_boss: true })
      else if (k === Qt.Key_F3) root.cheat({ random_pack: true })
      else if (k === Qt.Key_F12) root.cheat({ random_command: true })
      else if (k === Qt.Key_F11) root.cheat({ enhance_hand: true })
      else if (k === Qt.Key_F10) root.cheat({ random_theme: true })
      else if (k === Qt.Key_F9) root.cheat({ random_joker: true })
      else if (k === Qt.Key_F8) root.cheat({ level_up: root.game.hands.map(function(h) { return h.kind }) })
      else if (k === Qt.Key_Return || k === Qt.Key_Enter || k === Qt.Key_Space) root.confirm()
      else if (k === Qt.Key_D || k === Qt.Key_Delete) root.discard()
      else if (k === Qt.Key_N) root.newGame()
      else return
      event.accepted = true
    }

    // Debug only (OMATRO_DEBUG=1): plays the steps of OMATRO_SCRIPT, separated
    // by ";": "key:Return", "key:Shift+Left", "key:Ctrl+1", "sleep:500",
    // "shot:/path/name.png" (the window, as rendered), "size:1100,700", "quit",
    // "audio" (scripted runs are silent until then).
    // Screenshots this way need no keyboard, pointer or focus, even offscreen.
    property var scriptSteps: []
    Timer {
      id: scriptTimer
      interval: 300
      onTriggered: keyCatcher.runScript()
    }
    Component.onCompleted: {
      var script = engine.script()
      if (script === "") return
      scriptSteps = script.split(";").map(function(s) { return s.trim() }).filter(function(s) { return s !== "" })
      scriptTimer.start()
    }
    function runScript() {
      if (!scriptSteps.length) return
      var step = scriptSteps[0]
      scriptSteps = scriptSteps.slice(1)
      var colon = step.indexOf(":")
      var verb = colon < 0 ? step : step.slice(0, colon)
      var arg = colon < 0 ? "" : step.slice(colon + 1)
      var wait = 300
      if (verb === "key") {
        var parts = arg.split("+")
        var modifiers = 0
        for (var i = 0; i < parts.length - 1; i++)
          modifiers |= parts[i] === "Shift" ? Qt.ShiftModifier : parts[i] === "Ctrl" ? Qt.ControlModifier : 0
        var key = Qt["Key_" + parts[parts.length - 1]]
        if (key === undefined) console.warn("script: unknown key", arg)
        else pressKey({ key: key, modifiers: modifiers, accepted: false })
      }
      else if (verb === "sleep") wait = parseInt(arg)
      else if (verb === "shot") {
        root.contentItem.grabToImage(function(result) {
          if (!result.saveToFile(arg)) console.warn("script: couldn't save", arg)
        })
      }
      else if (verb === "size") {
        var wh = arg.split(",")
        root.width = parseInt(wh[0])
        root.height = parseInt(wh[1])
      }
      else if (verb === "quit") Qt.quit()
      else if (verb === "audio") root.scriptAudio = true
      else console.warn("script: unknown step", step)
      scriptTimer.interval = wait
      scriptTimer.start()
    }

    RowLayout {
      anchors { fill: parent; margins: Style.spacing.huge }
      spacing: Style.spacing.huge

      // ---- left column: blind, score, running calculation
      // Fixed width: a nested layout stretches by default and the column
      // would resize with the right-hand content (game or shop).
      ColumnLayout {
        Layout.fillWidth: false
        Layout.preferredWidth: Style.space(250)
        Layout.minimumWidth: Style.space(250)
        Layout.maximumWidth: Style.space(250)
        Layout.fillHeight: true
        spacing: Style.spacing.xxl

        Column {
          Text {
            text: "omatro"
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.displayLarge
            font.bold: true
          }
          // The machine the game is played on.
          Caption {
            visible: root.game !== null && root.game.run_deck !== null
            text: visible ? I18n.tr("on {deck}", { deck: root.game.run_deck.name }) + " · " + root.game.stake.name : ""
          }
        }

        Box {
          Layout.fillWidth: true
          implicitHeight: blindCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: blindCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.xs
            Caption {
              text: root.game
                ? (root.endless ? I18n.tr("Ante {n}", { n: root.game.ante })
                    : I18n.tr("Ante {a} / {b}", { a: root.game.ante, b: root.game.winning_ante }))
                  + " · " + (root.currentBlind ? root.currentBlind.name : "")
                : I18n.tr("Ante")
            }
            Value { text: root.game ? Labels.number(root.game.target) : "–" }
            // The ante's boss, ahead of its blind (muted) and while its effect
            // applies (urgent).
            Text {
              width: parent.width
              visible: root.upcomingBoss
              wrapMode: Text.WordWrap
              textFormat: Text.StyledText
              text: visible
                ? "<b>" + I18n.tr("Boss: {name}", { name: root.game.boss.name }) + "</b><br>" + root.game.boss.description
                : ""
              color: root.bossName !== "" && root.game.phase === "playing" ? Colors.urgent : Colors.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
          }
        }

        Box {
          Layout.fillWidth: true
          implicitHeight: scoreCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: scoreCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.xs
            Caption { text: I18n.tr("Blind score") }
            Value { text: Labels.number(root.shownScore) }
          }
        }

        Box {
          Layout.fillWidth: true
          implicitHeight: moneyCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: moneyCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.xs
            Caption { text: I18n.tr("Money") }
            Row {
              spacing: Style.spacing.lg
              Value { text: root.game ? Labels.money(Math.round(root.shownMoney)) : "–"; color: Colors.accent }
              // The last change, rising and fading.
              Text {
                id: moneyDelta
                anchors.baseline: parent.children[0].baseline
                opacity: 0
                font.family: Style.font.family
                font.pixelSize: Style.font.title
                font.bold: true
                transform: Translate { id: moneyRise }
                function show(delta) {
                  text = (delta > 0 ? "+" : "−") + Labels.money(Math.abs(delta))
                  color = delta > 0 ? Colors.accent : Colors.urgent
                  moneyFloat.restart()
                }
                ParallelAnimation {
                  id: moneyFloat
                  NumberAnimation { target: moneyDelta; property: "opacity"; from: 1; to: 0; duration: 1200; easing.type: Easing.InQuad }
                  NumberAnimation { target: moneyRise; property: "y"; from: 0; to: -Style.space(14); duration: 1200; easing.type: Easing.OutCubic }
                }
              }
            }
          }
        }

        // Recognized hand and chips × mult (preview, then animated scoring),
        // on fire when a single hand beats the blind.
        Box {
          Layout.fillWidth: true
          implicitHeight: calcCol.implicitHeight + 2 * Style.spacing.xxl
          border.color: root.fireHeat > 0 ? Colors.warm : Style.normalBorderColor
          Behavior on border.color { ColorAnimation { duration: 300 } }
          // Behind the hand name and the counters.
          ScoreFire {
            anchors { fill: parent; margins: Style.normalBorderWidth }
            heat: root.fireHeat
          }
          Column {
            id: calcCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.md
            // The hand and its level.
            Row {
              width: parent.width
              spacing: Style.spacing.md
              Text {
                id: handNameText
                text: root.handName || I18n.tr("Choose cards")
                color: root.handName ? Colors.foreground : Colors.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.heading
                font.bold: root.handName !== ""
              }
              Text {
                anchors.baseline: handNameText.baseline
                visible: root.handLevel > 0
                text: I18n.tr("lvl.{n}", { n: root.handLevel })
                color: Colors.accent
                font.family: Style.font.family
                font.pixelSize: Style.font.body
                font.bold: true
              }
            }
            Row {
              spacing: Style.spacing.md
              Counter {
                id: chipsCounter
                tint: Colors.accent
                value: Labels.number(root.liveChips)
              }
              Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: Colors.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.heading
              }
              Counter {
                id: multCounter
                tint: Colors.urgent
                value: Labels.mult(root.liveMult)
              }
            }
          }
        }

        RowLayout {
          Layout.fillWidth: true
          spacing: Style.spacing.xxl
          Box {
            Layout.fillWidth: true
            implicitHeight: handsCol.implicitHeight + 2 * Style.spacing.xl
            Column {
              id: handsCol
              anchors { fill: parent; margins: Style.spacing.xl }
              Caption { text: I18n.tr("Hands") }
              Value { text: root.game ? root.game.hands_left : "–"; color: Colors.accent }
            }
          }
          Box {
            Layout.fillWidth: true
            implicitHeight: discardsCol.implicitHeight + 2 * Style.spacing.xl
            Column {
              id: discardsCol
              anchors { fill: parent; margins: Style.spacing.xl }
              Caption { text: I18n.tr("Discards") }
              Value { text: root.game ? root.game.discards_left : "–"; color: Colors.urgent }
            }
          }
        }

        Caption { text: root.game ? I18n.tr("Deck: {a} / {b}", { a: root.game.remaining.length, b: root.game.deck_size }) : "" }

        // Badges won, waiting for their moment.
        Column {
          Layout.fillWidth: true
          visible: root.game !== null && root.game.tags.length > 0
          spacing: Style.spacing.sm
          Caption { text: I18n.tr("Badges") }
          Flow {
            width: parent.width
            spacing: Style.spacing.sm
            Repeater {
              model: root.game ? root.game.tags : []
              BadgeView { tag: modelData }
            }
          }
        }

        // Clipboard: consumables waiting to be used (Ctrl+1, Ctrl+2).
        Column {
          Layout.fillWidth: true
          spacing: Style.spacing.md
          Caption {
            // Transparent consumables take no slot.
            text: root.game ? I18n.tr("Clipboard {a} / {b}", {
              a: root.game.consumables.filter(function(c) { return !c.transparent }).length,
              b: root.game.consumable_slots }) : ""
          }
          // Past two cards (Transparent copies take no slot, but room), they
          // overlap: hover or click one to bring it to the front.
          Fan {
            id: clipboardRow
            width: implicitWidth
            height: implicitHeight
            count: root.game ? root.game.consumables.length : 0
            itemWidth: (Style.space(250) - spacing) / 2
            spacing: Style.spacing.md
            available: Style.space(250)
            Repeater {
              model: root.game ? root.game.consumables : []
              Column {
                x: clipboardRow.xOf(index)
                y: clipboardRow.liftOf(index)
                z: clipboardRow.zOf(index)
                Behavior on x { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
                Behavior on y { NumberAnimation { duration: 100 } }
                spacing: Style.spacing.sm
                // Its controls only show in front, when the cards overlap.
                readonly property bool controls: clipboardRow.showsControls(index)
                HoverHandler {
                  blocking: true
                  onHoveredChanged: clipboardRow.hover(index, hovered)
                }
                ConsumableView {
                  item: modelData
                  width: clipboardRow.itemWidth
                  TapHandler { onTapped: clipboardRow.pin(index) }
                }
                Caption {
                  width: clipboardRow.itemWidth
                  visible: modelData.max_cards > 0
                  opacity: parent.controls ? 1 : 0
                  fontSizeMode: Text.HorizontalFit
                  minimumPixelSize: Style.fontPx(0.6)
                  horizontalAlignment: Text.AlignHCenter
                  text: I18n.tr("select {range}", { range: Labels.range(modelData.min_cards, modelData.max_cards) })
                }
                Row {
                  anchors.horizontalCenter: parent.horizontalCenter
                  opacity: parent.controls ? 1 : 0
                  enabled: parent.controls
                  spacing: Style.spacing.xs
                  Button {
                    compact: true
                    text: I18n.tr("Apply")
                    enabled: root.canUse(modelData)
                    opacity: enabled ? 1 : 0.5
                    onClicked: root.useConsumable(index)
                  }
                  Button {
                    compact: true
                    text: Labels.money(modelData.sell_value)
                    enabled: !root.busy && root.game.phase !== "lost"
                    onClicked: root.sellConsumable(index)
                  }
                }
              }
            }
          }
        }

        Item { Layout.fillHeight: true }

        Caption {
          Layout.fillWidth: true
          wrapMode: Text.WordWrap
          text: root.shopping
            ? I18n.tr("1-9: buy · Shift+1-9: use now\nR: reroll · Enter: next blind\nJ, Shift+←/→: move a joker\nI: fastfetch · C: collection · Ctrl+Q: quit")
            : root.inPack
            ? I18n.tr(root.packHand ? "1-9, 0: select cards · Shift+1-5: install\n" : "1-5: install\n") + I18n.tr("Ctrl+C: skip · I: fastfetch\nC: collection · Ctrl+Q: quit")
            : root.choosing
            ? I18n.tr("Enter: play the blind · S: skip it\n") + (root.game.boss_reroll_cost !== null ? I18n.tr("B: reroll the boss · ") : "")
              + I18n.tr("N: new game · C: collection\nI: fastfetch · Ctrl+Q: quit")
            : I18n.tr("1-9, 0: choose · Enter: play\nD: discard · Tab: sort\nShift+←/→: move a card\nJ, Shift+←/→: move a joker\nN: new game · I: fastfetch\nC: collection · Ctrl+Q: quit")
        }
      }

      // ---- play area: jokers, table, hand
      ColumnLayout {
        id: playArea
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: Style.spacing.huge

        Caption {
          Layout.alignment: Qt.AlignHCenter
          text: root.game ? I18n.tr("Jokers {a} / {b}", { a: root.game.jokers.length, b: root.game.max_jokers }) : ""
        }

        // Jokers overlap when they no longer fit (Transparent jokers add
        // slots): hover or click one, or press J, to bring it to the front.
        Fan {
          id: jokerRow
          Layout.alignment: Qt.AlignHCenter
          Layout.preferredWidth: implicitWidth
          Layout.preferredHeight: Style.space(230) + (root.jokerControls ? Style.space(40) : 0)
          count: root.game ? root.game.jokers.length : 0
          itemWidth: Style.space(120)
          spacing: Style.spacing.xxl
          available: root.playWidth
          Repeater {
            id: jokerRepeater
            model: root.game ? root.game.jokers : []
            Column {
              x: jokerRow.xOf(index)
              y: jokerRow.liftOf(index)
              z: jokerDragger.active ? jokerRow.count + 2 : jokerRow.zOf(index)
              Behavior on x { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
              Behavior on y { NumberAnimation { duration: 100 } }
              width: jokerRow.itemWidth
              spacing: Style.spacing.md
              // Dragged sideways, the joker moves in the row.
              transform: Translate { x: jokerDragger.active ? jokerDragger.activeTranslation.x : 0 }
              DragHandler {
                id: jokerDragger
                target: null
                enabled: root.jokersEditable
                yAxis.enabled: false
                cursorShape: Qt.ClosedHandCursor
                property real dragX: 0
                onActiveTranslationChanged: if (active) dragX = activeTranslation.x
                onActiveChanged: if (!active) {
                  root.moveJoker(index, index + Math.round(dragX / jokerRow.step))
                  dragX = 0
                }
              }
              // A joker that triggers comes to the front while it flashes.
              function flash(text, color) {
                jokerRow.flash(index)
                view.flash(text, color)
              }
              HoverHandler {
                blocking: true
                onHoveredChanged: jokerRow.hover(index, hovered)
              }
              JokerView {
                id: view
                joker: modelData
                width: jokerRow.itemWidth
                TapHandler { onTapped: jokerRow.pin(index) }
                hidden: root.game.jokers_hidden
                disabled: root.game.disabled_joker === index
                // The badge follows the selected hand (Muscle Memory).
                bonus: root.preview && root.preview.bonuses.length > index
                  ? root.preview.bonuses[index] : modelData.bonus
              }
              // Sell; only in front when the jokers overlap.
              Button {
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.jokerControls
                opacity: jokerRow.showsControls(index) ? 1 : 0
                enabled: root.jokersEditable && jokerRow.showsControls(index)
                compact: true
                text: I18n.tr("Sell {money}", { money: Labels.money(modelData.sell_value) })
                onClicked: root.sell(index)
              }
            }
          }
        }

        // Shop, between two blinds, in zones as at a counter: the buttons and,
        // under them, the pull requests; the offers and, under them, the
        // packs. Keys follow the order: offers, then packs.
        ColumnLayout {
          id: shop
          Layout.fillWidth: true
          Layout.fillHeight: true
          visible: root.game !== null && root.game.phase === "shop"
          spacing: Style.spacing.xxl
          readonly property real gap: Style.space(16)
          readonly property real zonePadding: 2 * Style.spacing.xxl
          readonly property real buttonsWidth: Style.space(190)
          // Items get narrower when a column has more of them (More
          // Workspaces, PRs: welcome).
          readonly property int vouchers: root.game ? root.game.vouchers_for_sale.length : 0
          readonly property int wide: root.game ? Math.max(1, root.game.shop.length, root.game.packs.length) : 1
          readonly property real itemWidth: {
            var room = root.playWidth - zonePadding - Style.spacing.huge - (wide - 1) * gap
            var fit = vouchers > 1
              ? (room - zonePadding - (vouchers - 1) * gap) / (wide + vouchers)
              : (room - buttonsWidth) / wide
            return Math.min(Style.space(120), fit)
          }
          // Offers are as tall as the tallest of them, so their buttons line up.
          readonly property real offerHeight: root.game && root.game.shop.some(function(o) { return o.kind === "joker" })
            ? Style.space(230) : Style.space(150)

          Item { Layout.fillHeight: true }

          Grid {
            Layout.alignment: Qt.AlignHCenter
            columns: 2
            columnSpacing: Style.spacing.huge
            rowSpacing: Style.spacing.huge

            // Next blind and reroll, centred on the offers.
            Item {
              width: Math.max(shop.buttonsWidth, pullRequests.visible ? pullRequests.width : 0)
              height: offersZone.height
              Column {
                anchors.centerIn: parent
                spacing: Style.spacing.lg
                Button {
                  width: shop.buttonsWidth
                  text: I18n.tr("Next blind (Enter)")
                  bordered: true
                  enabled: root.shopping
                  onClicked: root.confirm()
                }
                Button {
                  width: shop.buttonsWidth
                  text: I18n.tr("Reroll (R) · {money}", { money: root.game ? Labels.money(root.game.reroll_cost) : "" })
                  bordered: true
                  enabled: root.shopping && root.canPay(root.game.reroll_cost)
                  opacity: enabled ? 1 : 0.5
                  onClicked: root.reroll()
                }
              }
            }

            ShopZone {
              id: offersZone
              title: I18n.tr("Offers")
                Repeater {
                  model: root.game ? root.game.shop : []
                  Column {
                    width: shop.itemWidth
                    spacing: Style.spacing.md
                    Item {
                    width: shop.itemWidth
                    height: shop.offerHeight
                    JokerView {
                      visible: modelData.kind === "joker"
                      width: shop.itemWidth
                      joker: visible ? modelData : null
                      opacity: root.canBuy(modelData) ? 1 : 0.5
                    }
                    ConsumableView {
                      visible: modelData.kind !== "joker" && modelData.kind !== "card"
                      width: shop.itemWidth
                      item: visible ? modelData : null
                      opacity: root.canBuy(modelData) ? 1 : 0.5
                    }
                    CardView {
                      anchors.horizontalCenter: parent.horizontalCenter
                      visible: modelData.kind === "card"
                      width: Math.min(shop.itemWidth, Style.space(84))
                      card: visible ? modelData.card : null
                      modifiers: root.game ? root.game.modifiers : null
                      interactive: false
                      opacity: root.canBuy(modelData) ? 1 : 0.5
                    }
                    }
                    Column {
                      anchors.horizontalCenter: parent.horizontalCenter
                      spacing: Style.spacing.xs
                      Button {
                        anchors.horizontalCenter: parent.horizontalCenter
                        compact: true
                        text: root.keyName(index) + " · " + Labels.money(modelData.price)
                        bordered: true
                        enabled: root.canBuy(modelData)
                        opacity: enabled ? 1 : 0.5
                        onClicked: root.buy(index)
                      }
                      // Bought and used at once, without the clipboard.
                      Button {
                        anchors.horizontalCenter: parent.horizontalCenter
                        compact: true
                        visible: (modelData.kind === "theme" || modelData.kind === "command") && !modelData.needs_hand
                        text: "⇧" + root.keyName(index) + " · " + I18n.tr("Use now")
                        enabled: root.canBuyAndUse(modelData)
                        opacity: enabled ? 1 : 0.5
                        onClicked: root.buyAndUse(index)
                      }
                    }
                  }
                }
                Caption {
                  visible: root.game !== null && root.game.shop.length === 0
                  width: shop.itemWidth * 2
                  height: shop.offerHeight
                  verticalAlignment: Text.AlignVCenter
                  horizontalAlignment: Text.AlignHCenter
                  wrapMode: Text.WordWrap
                  text: I18n.tr("Sold out: reroll for new offers.")
                }
            }

            // The pull requests of the ante (numbered after the packs). The cell
            // stays when there are none, so the packs stay under the offers
            // (1 pixel: positioners skip empty items).
            Item {
              width: pullRequests.visible ? pullRequests.width : 1
              height: pullRequests.visible ? pullRequests.height : 1
              ShopZone {
                id: pullRequests
                title: I18n.tr("Pull request")
                visible: shop.vouchers > 0
                  Repeater {
                    model: root.game ? root.game.vouchers_for_sale : []
                    Column {
                      width: shop.itemWidth
                      spacing: Style.spacing.md
                      VoucherView {
                        width: shop.itemWidth
                        voucher: modelData
                        opacity: root.canMerge(index) ? 1 : 0.5
                      }
                      Button {
                        anchors.horizontalCenter: parent.horizontalCenter
                        compact: true
                        text: root.keyName(root.game.shop.length + root.game.packs.length + index)
                          + " · " + Labels.money(modelData.price)
                        bordered: true
                        enabled: root.canMerge(index)
                        opacity: enabled ? 1 : 0.5
                        onClicked: root.buyVoucher(index)
                      }
                    }
                  }
              }
            }

            // Packs, opened on the spot.
            ShopZone {
              title: I18n.tr("Packages")
              visible: root.game !== null && root.game.packs.length > 0
                Repeater {
                  model: root.game ? root.game.packs : []
                  Column {
                    width: shop.itemWidth
                    spacing: Style.spacing.md
                    PackView {
                      width: shop.itemWidth
                      pack: modelData
                      opacity: root.canBuyPack(modelData) ? 1 : 0.5
                    }
                    Button {
                      anchors.horizontalCenter: parent.horizontalCenter
                      compact: true
                      text: root.keyName(root.game.shop.length + index) + " · " + Labels.money(modelData.price)
                      bordered: true
                      enabled: root.canBuyPack(modelData)
                      opacity: enabled ? 1 : 0.5
                      onClicked: root.buyPack(index)
                    }
                  }
                }
            }
          }

          Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.notice
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }

          Item { Layout.fillHeight: true }
        }

        // Choice of the next blind.
        ColumnLayout {
          Layout.fillWidth: true
          Layout.fillHeight: true
          visible: root.choosing
          spacing: Style.spacing.huge

          Item { Layout.fillHeight: true }

          BlindSelect {
            Layout.alignment: Qt.AlignHCenter
            blinds: root.game ? root.game.blinds : []
            bossRerollCost: root.game && root.game.boss_reroll_cost !== null ? root.game.boss_reroll_cost : -1
            onPlay: root.confirm()
            onSkip: root.skipBlind()
            onRerollBoss: root.rerollBoss()
          }

          Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.notice
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }

          Item { Layout.fillHeight: true }
        }

        // An open pack.
        ColumnLayout {
          Layout.fillWidth: true
          Layout.fillHeight: true
          visible: root.inPack
          Item { Layout.fillHeight: true }
          PackOpening {
            Layout.alignment: Qt.AlignHCenter
            opened: root.inPack ? root.game.opened : null
            modifiers: root.game ? root.game.modifiers : null
            canPick: root.canPick
            keyName: function(i) { return (root.packHand ? "⇧" : "") + root.keyName(i) }
            onPick: function(index) { root.pickChoice(index) }
            onSkip: root.skipPack()
          }
          Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.notice
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }
          Item { Layout.fillHeight: true }
        }

        // Table: cards being scored, or the last result.
        Item {
          Layout.fillWidth: true
          Layout.fillHeight: true
          visible: !root.shopping && !root.choosing && !root.inPack

          Row {
            anchors.centerIn: parent
            spacing: Style.spacing.xxl
            visible: root.scoring !== null
            Repeater {
              id: tableRepeater
              model: root.scoring ? root.scoring.cards : []
              CardView {
                modifiers: root.game ? root.game.modifiers : null
                card: modelData
                interactive: false
                dimmed: root.scoring !== null && root.scoring.scoring.indexOf(index) < 0
                disabledBy: root.scoring !== null && root.scoring.steps.some(function(s) {
                  return s.effect === "debuffed" && s.source.kind === "card" && s.source.index === index
                }) ? root.game.boss.name : ""
              }
            }
          }

          Text {
            anchors { horizontalCenter: parent.horizontalCenter; bottom: parent.bottom }
            visible: root.showTotal
            // Pops in.
            onVisibleChanged: if (visible) totalPop.restart()
            NumberAnimation on scale {
              id: totalPop
              running: false
              from: 1.6; to: 1; duration: 350; easing.type: Easing.OutBack
            }
            text: root.scoring ? "= " + Labels.number(root.scoring.total) : ""
            color: Colors.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.displayLarge
            font.bold: true
          }

          Text {
            anchors.centerIn: parent
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            visible: root.scoring === null
            text: root.notice
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }
        }

        Row {
          id: handRow
          Layout.alignment: Qt.AlignHCenter
          Layout.topMargin: Style.space(24)
          visible: !root.shopping && !root.choosing && (!root.inPack || root.packHand)
          spacing: Style.spacing.lg
          // Narrower cards for a bigger hand (Ultrawide).
          readonly property int count: Math.max(8, root.game ? root.game.hand_size : 8)
          readonly property real cardWidth: Math.min(Style.space(84),
            (root.playWidth - (count - 1) * spacing) / count)
          Repeater {
            id: handRepeater
            model: root.hand
            CardView {
                modifiers: root.game ? root.game.modifiers : null
              width: handRow.cardWidth
              card: modelData
              selected: root.selected.indexOf(index) >= 0
              interactive: root.selecting
              faceDown: root.faceDownIds.indexOf(modelData.id) >= 0
              disabledBy: root.debuffedIds.indexOf(modelData.id) >= 0 ? root.bossName : ""
              forced: modelData.id === root.forcedId
              shortcut: root.keyName(index)
              onClicked: root.toggleCard(index)
              // Dragged sideways, the card moves in the hand.
              property real dragX: 0
              z: dragger.active ? 10 : 0
              transform: Translate { x: dragger.active ? dragger.activeTranslation.x : 0 }
              DragHandler {
                id: dragger
                target: null
                enabled: root.selecting
                yAxis.enabled: false
                cursorShape: Qt.ClosedHandCursor
                onActiveTranslationChanged: if (active) parent.dragX = activeTranslation.x
                onActiveChanged: if (!active) {
                  root.moveCard(index, index + Math.round(parent.dragX / (handRow.cardWidth + handRow.spacing)))
                  parent.dragX = 0
                }
              }
            }
          }
        }

        Row {
          Layout.alignment: Qt.AlignHCenter
          Layout.topMargin: Style.space(16)
          visible: !root.shopping && !root.choosing && !root.inPack
          spacing: Style.spacing.xxl
          Button {
            text: I18n.tr("Play hand")
            bordered: true
            enabled: root.playing && root.selected.length > 0
            opacity: enabled ? 1 : 0.5
            onClicked: root.play()
          }
          Button {
            text: I18n.tr("Discard")
            bordered: true
            enabled: root.playing && root.selected.length > 0 && root.game.discards_left > 0
            opacity: enabled ? 1 : 0.5
            onClicked: root.discard()
          }
          // Sort the hand (Tab switches); drag a card to move it.
          Row {
            anchors.verticalCenter: parent.verticalCenter
            spacing: Style.spacing.md
            Caption {
              anchors.verticalCenter: parent.verticalCenter
              text: I18n.tr("Sort (Tab)")
            }
            Repeater {
              model: [["rank", "Rank"], ["suit", "Suit"]]
              Button {
                compact: true
                text: I18n.tr(modelData[1])
                bordered: root.game !== null && root.game.sort_by === modelData[0]
                enabled: root.selecting
                opacity: enabled ? 1 : 0.5
                onClicked: root.sortHand(modelData[0])
              }
            }
          }
        }
      }
    }

    // ---- end of blind: won or lost
    Rectangle {
      anchors.fill: parent
      visible: root.game !== null && (root.game.phase === "won" || root.game.phase === "lost") && !root.busy
      color: Util.alpha(Colors.background, 0.85)

      MouseArea { anchors.fill: parent }

      Box {
        anchors.centerIn: parent
        width: Style.space(420)
        height: endCol.implicitHeight + 2 * Style.spacing.huge
        color: Colors.background
        border.color: root.game && root.game.phase === "won" ? Colors.accent : Colors.urgent

        Column {
          id: endCol
          anchors { fill: parent; margins: Style.spacing.huge }
          spacing: Style.spacing.xxl
          readonly property bool won: root.game !== null && root.game.phase === "won"
          readonly property bool victory: endCol.won && root.game.victory

          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: I18n.tr(endCol.victory ? "Omarchy achieved!" : endCol.won ? "Blind won!" : "Game over")
            color: endCol.won ? Colors.accent : Colors.urgent
            font.family: Style.font.family
            font.pixelSize: Style.font.displayLarge
            font.bold: true
          }
          Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            visible: endCol.victory || !endCol.won
            text: endCol.victory
              ? I18n.tr("Ante {n} beaten. Go post your setup on r/unixporn.", { n: root.game.winning_ante })
              : I18n.tr("Have you tried rebooting?")
            color: Colors.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.body
            font.italic: true
          }
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.game ? Labels.number(root.game.score) + " / " + Labels.number(root.game.target)
              + (endCol.won ? "" : " · ante " + root.game.ante) : ""
            color: Colors.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }

          Text {
            width: parent.width
            visible: endCol.won && root.game.reward !== null && root.game.reward.notes.length > 0
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: visible ? root.game.reward.notes.join(" ") : ""
            color: Colors.urgent
            font.family: Style.font.family
            font.pixelSize: Style.font.body
          }

          // Reward breakdown.
          Grid {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: endCol.won && root.game.reward !== null
            columns: 2
            columnSpacing: Style.space(40)
            rowSpacing: Style.spacing.sm
            Repeater {
              model: {
                var r = root.game && root.game.reward
                if (!r) return []
                var lines = [[I18n.tr("Blind"), r.blind], [I18n.tr("Hands left"), r.hands], [I18n.tr("Interest"), r.interest]]
                if (r.jokers > 0) lines.push([I18n.tr("Jokers"), r.jokers])
                if (r.cards > 0) lines.push([I18n.tr("Enterprise cards"), r.cards])
                if (r.badges > 0) lines.push([I18n.tr("Badges"), r.badges])
                if (r.discards > 0) lines.push([I18n.tr("Discards left"), r.discards])
                lines.push([I18n.tr("Total"), r.total])
                var cells = []
                for (var i = 0; i < lines.length; i++)
                  cells.push({ text: lines[i][0], value: false, last: i === lines.length - 1 },
                             { text: "+" + Labels.money(lines[i][1]), value: true, last: i === lines.length - 1 })
                return cells
              }
              Text {
                text: modelData.text
                color: modelData.value ? Colors.accent : (modelData.last ? Colors.foreground : Colors.muted)
                font.family: Style.font.family
                font.pixelSize: Style.font.subtitle
                font.bold: modelData.last
              }
            }
          }

          Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: Style.spacing.xxl
            Button {
              text: I18n.tr(endCol.victory ? "Endless mode (Enter)" : endCol.won ? "Cash out (Enter)" : "New game (Enter)")
              bordered: true
              onClicked: root.confirm()
            }
            Button {
              visible: endCol.victory
              text: I18n.tr("New game (N)")
              bordered: true
              onClicked: root.newGame()
            }
          }

          // To replay it, or share it.
          SeedLabel {
            id: endSeed
            anchors.horizontalCenter: parent.horizontalCenter
            visible: endCol.victory || !endCol.won
            code: root.game ? root.game.seed_code : ""
            shortcut: endCol.won ? "" : "Ctrl+C"
            pixelSize: Style.font.caption
          }
        }
      }
    }

    // Sparks of the scoring steps, over the game but under the panels.
    Sparks {
      id: sparks
      anchors.fill: parent
    }

    // ---- fastfetch: run info over everything
    FastFetch {
      id: fastFetch
      anchors.fill: parent
      visible: root.infoOpen
      game: root.game
      onClose: root.infoOpen = false
    }

    // ---- new game: the deck to play with
    NewGame {
      id: newGameScreen
      anchors.fill: parent
      visible: root.choosingDeck
      decks: root.decks
      stakes: root.stakes
      message: root.saveProblem
      onLanguage: function(lang) { root.setLanguage(lang) }
      music: settings.music
      musicVolume: settings.musicVolume
      onToggleMusic: root.toggleMusic()
      onMusicVolumeSet: function(v) { root.setMusicVolume(v) }
      sounds: settings.sounds
      soundVolume: settings.soundVolume
      onToggleSounds: root.toggleSounds()
      onSoundVolumeSet: function(v) { root.setSoundVolume(v) }
      canCancel: root.game !== null
      onStart: function(deck, stake, seed) { root.startGame(deck, stake, seed) }
      onSeedDone: keyCatcher.forceActiveFocus()
      onCancel: root.choosingDeck = false
      onCollection: root.openCollection()
    }

    // ---- the collection and the achievements
    Collection {
      id: collectionScreen
      anchors.fill: parent
      visible: root.collectionOpen
      collection: root.collection
      onClose: root.collectionOpen = false
    }

    // ---- news: items found, achievements done, what they unlocked
    Column {
      anchors { top: parent.top; right: parent.right; margins: Style.spacing.huge }
      spacing: Style.spacing.md
      Repeater {
        model: root.news
        Rectangle {
          readonly property bool feat: modelData.kind === "achievement" || modelData.kind === "unlocked"
          anchors.right: parent.right
          width: newsText.implicitWidth + 2 * Style.spacing.xl
          height: newsText.implicitHeight + 2 * Style.spacing.lg
          radius: Style.cornerRadius
          color: Qt.tint(Colors.background, Util.alpha(feat ? Colors.warm : Colors.accent, 0.15))
          border.width: Style.space(2)
          border.color: feat ? Colors.warm : Colors.accent
          Column {
            id: newsText
            anchors.centerIn: parent
            spacing: Style.spacing.xs
            Text {
              text: I18n.tr(modelData.kind === "achievement" ? "Achievement unlocked"
                : modelData.kind === "unlocked" ? "Now in your games"
                : modelData.kind === "music" ? "Now playing" : "New in the collection")
              color: Colors.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
            Text {
              text: modelData.name + (modelData.text ? " · " + modelData.text : "")
              color: feat ? Colors.warm : Colors.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              font.bold: true
            }
          }
        }
      }
    }

    // ---- at start: resume the game left in progress
    ResumeGame {
      anchors.fill: parent
      visible: root.resuming
      saved: root.savedGame
      onResume: root.resumeGame()
      onNewGame: root.declineResume()
    }

    // What the terms of the hovered card mean, over everything.
    Glossary {
      anchors.fill: parent
    }
  }
}
