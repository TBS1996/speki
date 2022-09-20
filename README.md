
# flash-tui
flashcard app in your terminal

[![Watch the video](https://i.imgur.com/njEoYNL.png)](https://youtu.be/hV1iETM6T8g)


## Installation  
 
I'll add this to various linux package databases once I've figured out what name I wanna give to this project. But for now, if you wanna use it you can do the following:  
 
- make sure rust and git is installed
- in your terminal, navigate to where you wanna download this project 
- enter following command: "git clone https://github.com/TBS1996/flash-tui"
- in the flash-tui directory, enter "cargo run"

If you have any questions, feel free to open up an issue!


## Keybindings

### General

Next tab: **Tab**  
Previous tab: **Shift+Tab**  
Switch between widgets: **Alt + (h | j | k | l)**  (or alt+arrowkeys)   
Quit: **Alt+q**  

### Review tab  

Skip card or text: **Alt+s**  
Mark unfinished card as finished: **Alt+f**  
On a text, make a new card with text as source: **Alt+a**  
Add existing card as dependency: **Alt+y**  
Add existing card as dependent: **Alt+t**  
Add new card as dependency: **Alt+Y**  
Add new card as dependent: **Alt+T**  
Rate your recall grade: **1-4**   
    	1. No recall even after you see the answer  
	2. Failed recall when presented answer, but answer is familiar  
	3. Correct recall after some time to think   
	4. Instant correct recall  

### Add card tab  
Add card as unfinished: **Alt+u**  
Add card as finished: **Alt+f**  

### Incremental reading tab  

Add new text under selected topic: **Alt+a**  

### Widget-specific keybindings  
 
#### Topic list  


up/down: **k/j**    
Place topic below topic beneath: **l**  
Put topic one step up: **h**  
move topic up/down: **K/J**  
  
#### Incremental reading widget  

normal mode: **Ctrl+c**  
visual mode from normal mode: **v**    
insert mode from normal mode: **i**    
new extract from visual mode: **Alt+x**    
new cloze from visual mode: **Alt+z**  
scroll down: **Ctrl+d**   
scroll up: **Ctrl+u**  
.... and a bunch of other vim commands, feel free to make pull requests for new ones  
 



## Known issues  
 
Text editing can be a bit buggy, especially when non-ascii characters are involved. The library I'm using doesn't include a way to edit text so I had to do it all manually.  












