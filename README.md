# Speki


Flashcards for your terminal



https://user-images.githubusercontent.com/56874491/196013353-846df752-7587-4b63-b0fa-416de64cdb1b.mp4




## Installation

### Windows  

[Click to download binary](https://github.com/TBS1996/speki/releases/download/v0.1.11/speki.exe)

It will probably be flagged as dangerous by windows


### AUR

Package is simply named "speki" in the aur. Can be found [here](https://aur.archlinux.org/packages/speki).  

if you're using yay:

```bash
yay -S speki
```


### Cargo

For something that will work on any distro as long as you have cargo installed, try this:  
  
```bash
cargo install speki
```
   
Note that this won't install all the necessary dependencies, for example look at the note on ubuntu right below.
You can get cargo by [installing rust](https://www.rust-lang.org/tools/install)

### Ubuntu

If you get the following error on ubuntu: 
`--- stderr thread 'main' panicked at '"pkg-config" "--libs" "--cflags" "alsa" did not exit successfully: exit status: 1 error: could not find system library 'alsa' required by the 'alsa-sys' crate `

Try installing `libasound2-dev` 
If you get the same error on another distro, try finding the equivalent to that dependency and installing it.


## Features

It's not just about having flashcards in your terminal


### Anki-integration  

Speki has integrated the shared decks from anki, meaning you can jump straight in to learning different things, all without even leaving the app. It also allows you to customize the templates before importing. Soon I will add the possibility to add your local anki-files so that you'll get your anki-cards together with their learning-history to Speki.


### Unfinished-cards 

Have a question but you're not able or don't have the time to find the answer to it? You can add unfinished cards in Speki, and be periodically asked if you're able to find the answer to them. No more needing a separate note-taking system for things you want to turn into flashcards.

### Knowledge dependencies

You have the answer to your question, but you don't understand the answer? You can add the card as finished, but add *dependencies* to it, which would be other cards you need to understand for the current card to make sense. 

For example, if you read this and you want to memorize it:  
`Alpha particles consist of two protons and two neutrons`

Let's pretend you don't know what protons or neutrons are. If you were to memorize this sentence in that case, the knowledge would be completely useless to you. In speki however, you can add the two following cards as dependencies:   
`What is a proton?`  
`What is a neutron?`  
  
If you add them as dependencies, the original card will be marked as **unresolved**, meaning it has unfinished dependencies. When it is unresolved, you will not be asked to answer it, but you'll be asked to learn what a proton and neutron is, when you've figured out their meanings, the alpha particles card will become **resolved** and you can review it as normal. This system ensures that no matter how complicated things you add, you'll never end up memorizing something you don't understand, as long as you mark your dependencies correctly. 

It also works recursively, meaning in the previous example, if you found the answer to "what is a proton" but didn't understand it, that card would itself have dependencies. The dependency-graph could go on indefinitively, but if you keep on the dependencies would eventually all resolve  and you'd finally learn the top-most card. This closely mirrors how your brain works, as everything you know is dependent on other pieces of knowledge. It also eliminates the need for making mind maps, as this is basically a more accurate type of mind maps.

### Incremental reading

Incremental reading is a concept originating in SuperMemo. It allows you to incrementally work your way through countless longform-texts in parallel, where you'll learn and memorize everything  you want from it. This is done by by being presented with the texts within speki, and every time you see an interesting piece, you take extracts from it. As if you were going through a real book and using a highlighter. Later you'll be presented with just the extract, and here you can choose to make further extracts, or directly making cloze deletions by marking the things you want to remember. When you read a non-fiction book, not long after you won't remember much from it. If you do it through incremental reading, you'll remember every single thing you choose to remember, albeit with some extra efforts. 

If you want to read more about incremental reading, you can check out [this](https://www.masterhowtolearn.com/2019-08-06-supermemos-incremental-reading-explained/) blog article, as well as many other articles from the same site. 

### OpenAI integration  

Got a key to OpenAI? Then you can autogenerate the answers to questions



https://user-images.githubusercontent.com/56874491/196179030-eadbbe51-9b7a-4334-a2dc-5afb004031d5.mp4




## Keyboard-shortcuts

### Universal

switch between tabs `tab/shift+tab`  
switch between widgets: `Alt+(h|j|k|l)`  
quit: `Alt+q`  
toggle help menu: `F1`  


### Per tab

#### Review
   
generate answer (if gpt3 key in config) `Alt+g`  
skip unfinished card `Alt+s`  
mark unfinished card as finished `Alt+f`  
suspend card `Alt+i`  
mark incremental text as done `Alt+d`  
skip incremental text `Alt+s`  
rate recall grade (worst..best) `1..4`  
add new card as dependency `Alt+Y`  
add existing card as dependency `Alt+y`  
add new card as dependent `Alt+T`  
add existing card as dependent `Alt+t`  
  
#### Add card

generate answer (if gpt3 key in config) `Alt+g`  
add card as finished `Alt+f`  
add card as unfinished `Alt+u`  

#### incremental reading 

add wikipedia page (from sources or extracts list) `Alt+w`  

#### import 

download deck that you've revealed description of `Enter`  
switch front and back template `Alt+s`  
change which card to preview (on preview button) `h/l`  


### Per widget

#### Textinput

insert-mode -> normal-mode `Ctrl+c`  
normal-mode -> insert-mode `i`  
normal-mode -> visual-mode `v`  
visual-mode -> normal-mode `Ctrl+c`  
page-up -> `Ctrl+u`  
page-down -> `Ctrl+d`  
extract (from visual mode) `Alt+x`  
cloze-deletion (from visual mode) `Alt+z`  
delete right of line (from command mode) `D`  

#### Topics

move cursor up/down `k/j`  
move topic up/down `K/J`  
delete topic `Del`  
edit topic name `e`  
add new child topic `a`  
move topic up the hiarchy `h`  
move topic below topic under `l`  

## Glossary  

  
Pending card `Card has not yet been activated`  
Unfinished card `Card does not have an answer`  
Finished card `Card has a proper answer`  
Unresolved card `One or more dependencies of the card is either unfinished or unresolved`  
Incremental reading `A way of converting longform texts into flashcards`  
Suspended card `Card won't show up in the learning process`  
  

## Future plans

* Add tab where you can browse and filter all cards and perform actions on them
* Add image suport for terminals that enable this
* Machine-Learning algorithm for spaced repetition
* Visualize dependency graph



## Feedback/Contributions

I'm very open to any feedback or PRs, feel free to open an issue if there's anything you're wondering, and i'll make sure to get back to you!

