# reversi-druid
A take on REVERSI table top game written with rust GUI framework Druid.

Application starts with 2 pairs of diagonally positioned white and black checkers in the center as dictated by the rules.
Application enters in pvp mode: buttons on the side can be used to switch the mode.


Score is shown in top left corner during the game.

PvP mode:
  Players change turns
  Players have their tiles animated
  When there is no way for current player to make a move - winning screen is displayed and game is reset.
  
PvE mode:
   Play vs Computer
   Harder levels of PvE take better decisions
   Players turn is not animated (For distinguishablility of turns made by computer)
   Computer turn is animated

![image](https://user-images.githubusercontent.com/100690036/156379872-0e2132e7-c0c5-4ec6-87de-907ecb2189d1.png)
![image](https://user-images.githubusercontent.com/100690036/156380657-236caa90-28f3-46eb-890d-b4e4c18cc91f.png)
![image](https://user-images.githubusercontent.com/100690036/156380722-37f75175-7491-47b6-98f4-dfff0665453c.png)
![image](https://user-images.githubusercontent.com/100690036/156380816-9fe9c627-759a-4c57-9014-a1a489ab0c07.png)
![image](https://user-images.githubusercontent.com/100690036/156380884-9060379c-325d-463d-b211-8095b3b05d06.png)


