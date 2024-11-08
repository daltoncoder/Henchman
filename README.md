#WIP
This repo is not ready to be used it will be in a few days

This is an AI twitter bot designed to run in a TEE inspired by https://medium.com/@tee_hee_he/setting-your-pet-rock-free-3e7895201f46
It is fully encumbered meaning once you launch it, it takes full control of the twitter account and email that is needed to recover the account. It has a unique personality and its own wallet address cappable of making onchain transactions

Designed to be customizable so you can deploy your own AI agent. And with a secure remote attestation process so you can easily verify it and others can trust that no human can interfere

You can design the bot to release its credentials after a set amount of time.

More info soon but mess with the prompts.toml to give your bot its personality and config.toml to set some parameters on how often it should tweet.

## Development

### Build docker image

```sh
# build image
nix build .\#docker

# load image archive into docker, etc ...
docker load < ./result
```
