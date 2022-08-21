# flash-tui
flashcard app in your terminal

<img src="./media/demo.gif">

## Explanation  

Now you can work on your flashcards without leaving your terminal!  

Features dependency logic, where "card A" can have "card B" as a dependency. This means that you cannot understand card A without knowing card B.   
This allows for some interesting possibilities. For example, if you're reading the rust book, and you see the sentence "The box type stores data on the heap", but you don't know what a box type is, nor do you know what the heap is.   

You can still add this card on my app. Simply add it as usual like this:  
Q: Where does the Box type store data?  
A: On the heap.  

Then add the two following dependencies:    
Q: What is the Box type?  
A: *blank*  

Q: What is the heap?  
A: *blank*  
 
You will mark those cards as unfinished, as you do not yet know the answer to the questions. The original card will be de-activated, meaning it won't come up for review as it has unfinished dependencies. When you find out the answer to the dependencies, the original card will be activated. This also works recursively, you might find that the dependencies themselves have dependencies, it might go many layers deep, until you finally reach the end and all the cards will be activated as a line of falling dominos.
