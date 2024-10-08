# sandy
A simple example on how to execute sandwich attacks for Raydium AMM pools on the solana blockchain.

This project was created for educational purposes and is being open-sourced in the hope that it may help others.

#### Overview
This project consists of 2 parts: the [bot](./bot/) and the [program](./program/).

The bot is the off-chain client ran to find and execute sandwich oppportunities. It is responsible for reading transactions from the mempool, decoding the instruction data, then building & submitting the bundle.

The program is the on-chain program in which the bot interacts with to execute the front & back swaps of the sandwich attack. It is responsible for finding the optimal swap amount, executing the swaps, calculating the total profit, then tipping a percentage of that profit. I found that a custom program is required for the 2 main reasons of having the ability to calculate the profit of the bundle so we can tip accordingly, and being able to swap out all received tokens without knowing the amount before building the transaction.

#### Features
- on-chain tip calculation
- sandwich any swap that results in profit
- send bundles through jito's blockengine
- supports both SOL-TOKEN and TOKEN-SOL pairs
- dynamic & easily extendable instruction data decoder

#### Note

This is my first project related to MEV, and I’m sure there are many aspects that could be optimized or improved. Feel free to submit a pull request if you have any suggestions or enhancements to share.

#### Resources / Inspiration
- [subway](https://github.com/libevm/subway)
- [subway-rs](https://github.com/refcell/subway-rs)