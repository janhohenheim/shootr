# Contributing to shootr
The commit messages should be in accordance to [these common guidelines](https://github.com/erlang/otp/wiki/writing-good-commit-messages)  
TL;DR: Write them as a description of what happens to the codebase once they're applied.  
Bad: `Fixed style error`  
Good: `Fix style error in readme`

Aditionally, for organizatory purposes, it would be appreciated if you prefixed your commits to show which part of the project you've worked on. 

For example, if you fixed a typo in the readme file, the commit message should look like this:   
`Repo: Fix typo`

If you fixed a physics bug, a possible commit message would be:  
`Core: Fix character getting stuck in walls`

The possible prefixes are:
 - **Core**: Rust backend of the game
 - **Site**: Node backend of the site the game runs on
 - **Client**: Frontend of both the website and the game, e.g. rendering
 - **Repo**: Changes related to github, documentation, licensing, CI, etc.
 
 If you're uncertain what to pick: Don't worry, we will not deny your pull request if you used the wrong prefix :)

