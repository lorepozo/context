const {EventEmitter} = require('events')
const comb = require('./combinator')

class ProgramInductorEC extends EventEmitter {
  constructor(p) {
    this.name = 'programInductorEC'
    this.p = p // number of most probable combinators to keep
    this.on('iteration', this.run)
  }
  run(counter, ctx, sense) {
    const {getCombs, exploreCombs} = comb.fromContext(ctx)
    // TODO: EC yields a Map<comb, prob> learnedCombs from getCombs and sense
    const learnedCombs = new Map()
    const item = comb.findContainer(comb.mostProbable(learnedCombs), ctx.get())
    if (item !== null) {
      ctx.orient(item.id)
    }
    const probable = new Set(comb.nMostProbable(learnedCombs, this.p))
    const diff = new Set([...probable].filter(c => !exploreCombs.has(c)))
    if (diff.size > 0) {
      ctx.add(diff)
    }
  }
}

module.exports = ProgramInductorEC

