module.exports = {
  ink: (args) => console.log(args),
  random: ([min, max]) => Math.floor(Math.random() * (max - min + 1) + min),
  round: (num) => Math.round(num),
};
