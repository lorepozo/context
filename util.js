const {EventEmitter} = require('events')
export class Counter extends EventEmitter {
  constructor(total) {
    this.count = this.total = total
    this.hasFired = false
  }
  done() { // must be called total times to trigger 'done' event
    this.count--
    if (this.count <= 0 && !this.hasFired) {
      this.hasFired = true
      this.emit('done')
    }
  }
}
