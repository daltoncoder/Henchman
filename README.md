# Henchman TEE AI Agent

This is an AI twitter bot designed to run in a TEE inspired by https://medium.com/@tee_hee_he/setting-your-pet-rock-free-3e7895201f46
It is fully encumbered meaning once you launch it, it takes full control of the twitter account and email that is needed to recover the account. It has a unique personality and its own wallet address cappable of making onchain transactions

Designed to be customizable so you can deploy your own AI agent. And with a secure remote attestation process so you can easily verify it and others can trust that no human can interfere

You can design the bot to release its credentials after a set amount of time.

More info soon but mess with the prompts.toml to give your bot its personality and config.toml to set some parameters on how often it should tweet.

## Customizing the bot

There are two places you can easily customize the bot

### prompts.toml

The prompts.toml file in the root of this repo determines the bots personality and how often it tweets. The prompts.toml file included in this repo includes examples of what they could look like. It also is commented with the inputs the Agent adds into the prompts, make sure if you write your own prompts you include the spot these inputs should go. For example if the prompt includes `{short_term_memories}` input make sure you include the string "{short_term_memories}" in your prompt. The agent will replace that string with the actualy input before submitting it to the AI.

### config.toml

This file includes some required fields to fill out and also some parameters like how often the AI should be active, and how long it should have exclusive control of the account. After the lock period is over the AI will print the account details for you to take control of the account. If you plan on making this bot public, and verifiable you will have to push this data somewhere. It is safe to push the initial passwords to the account in this file publicly because the first thing the AI does is change the passwords and take control of the account. But do not use initial passwords that you use other places for this.

## Deploying

Accounts you need for the agent:

- Twitter Account- The one the Agent will own and tweet from
- Email account- The same one you used to sign up the twitter account. Currently bot only supports cock.li email accounts but this will change soon
- hyperbolic.xyz- You need an account here with an API key so we can query the AI models soon we will update to support more cloud AI providers
- OpenAI- You need a developer account here as we use their AI to create embeddings. We will soon update to only need 1 api key
  Once you have the twitter account, make sure you sign up for atleast free access to the Twitter API, and also if you generated an access key for the developer project make sure you revoke it before starting the bot or it will error out. The bot will generate the keys it needs when it starts

Once you have the accounts made fill out the config.toml with the account information

build the binary with nix so it can be reproducibly built and verified later

```sh
nix build .#tee-ai-agent --extra-experimental-features nix-command --extra-experimental-features flakes
```

then to deploy with gramine follow the readme in the enclave folder here
https://github.com/daltoncoder/Henchman/blob/main/enclave/README.md

After it is up and running on the TEE it will be waiting for you to provide the API keys it needs at port 6969 so make a post request with the keys like so

```sh
curl -X POST <IP_OF_AGENT>:6969/ --data '{"hyperbolic_api_key":"<YOUR_API_KEY>","open_ai_api_key":"<YOUR_API_KEY>"}'
```

Now your agent is started and fully autonomous until it releases its credentials

## Verifying a bot

The bot will host a remote attestation server on port 8000 and you can get its quote to do a remote attestation on it at `<IP>:8000/api/quote`
The userdata of the quote will contain the twitter username of the account it took control of, and it will not produce a quote until it has fully changed all passwords and taken control of the account. Even though the bot will not produce a quote until its taken control of the account, it will print its own MRENCLAVE as one of the first steps, even before the API keys are provided. So if you wanted someone to verify your Agent you could push up your config.toml and prompts.toml, they could reproducibly build the agent and check the logs for the MRENCLAVE to be able to verify your quote.
Step by step guide for doing a remote attestion coming soon.
