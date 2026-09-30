# Artist's three soulstones

Starting from the user-confirmed main-area arrival at **5530 / 0xB15BB212**,
steal all three items during one menu visit, then close the list and finish
the success dialogue. Predicted final state: **5535 / 0x94B3AD61**.
This theft's endpoint is source-derived and not yet live-confirmed.

`NPC_Twn_Cty_1_1_A_TALK_0900_D000` resolves to **Artist**. Its NPCBUY stock rows
01/02/03 contain Ice (`ITM_CSM_0320`), Wind (`ITM_CSM_0380`), and Light
(`ITM_CSM_0410`) Soulstones. Each has ProperSteal=-15, so level 2 exceeds the
100% threshold (difference 17 >= 11). Each attempt still consumes one CRT draw.
The placement has only an ordinary talk event, with no item/completion override.

| Boundary | Seed advances | CRT state |
| --- | ---: | --- |
| Open menu | 5531 | 0x5D96508D |
| First item obtained | 5532 | 0xC670211C |
| Second item obtained | 5533 | 0xD8F5AB6F |
| Third item obtained, menu still open | 5534 | 0xB91C1876 |
| Close menu and finish success dialogue | 5535 | 0x94B3AD61 |

`StealItemExec` returns IsListClose=false after ordinary successes, allowing
successive items without restarting the preview timer. Once the list closes,
Steal flow checks the accumulated GotItemList; with no override, state 2 calls
DrawEventLabel once for the whole session. The generic success events have
EncountReset=false. Thus +1 menu, +3 success rolls, +1 dialogue selection.
Item order does not change the total because all three are guaranteed successes.
See `throne-boy-steal.md` for the shared flow and script evidence.
