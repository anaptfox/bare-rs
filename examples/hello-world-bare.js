console.log('Hello from Bare-rs!');

// Example of async operation
setTimeout(() => {
  console.log('Async operation complete');
  Bare.exit(0);
}, 1000); 