const Network = require('./network')
const {Counter} = require('./util')
const config = require('./config')

class SKN {
  constructor({contextSize=5, maxSize=25, iterations=10}) {
    this.kn = new Network(arguments[0])
    this.mechanisms = new Set()
    this.T = iterations
  }
  register(mechanism) {
    this.mechanisms.add(mechanism)
  }
  run() {
    for (const t of [...Array(this.T).keys()]) {
      let counter = new Counter(this.mechanisms.size)
      let ctx = this.newContext(counter)
      this.mechanisms.forEach(mech => {
        mech.emit('iteration', counter, ctx, config.sensory[mech.name](t))
      })
    }
  }
  newContext(counter) {
    let callback = () => {} // TODO
    return this.kn.NewContext(counter, callback)
  }
}

// config.sensory[name] = function(iteration)

// mechanism extends EventEmitter
// mechanism has `name`
// mech.on('iteration', (counter, ctx, sense) => {...})

