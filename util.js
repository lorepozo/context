const {EventEmitter} = require('events')

// Counter emits a 'done' event once it has been triggered a
// specified number of times. A counter is triggered when the
// done() method is called.
export class Counter extends EventEmitter {
  constructor(total) {
    this.count = this.total = total
    this.hasFired = false
  }
  done() {
    this.count--
    if (this.count <= 0 && !this.hasFired) {
      this.hasFired = true
      this.emit('done')
    }
  }
}

export flip(p) {
  return Math.random() < p
}
