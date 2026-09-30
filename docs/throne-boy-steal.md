# Backstreets boy: Shadow Soulstone

Starting from the user-confirmed clothes-theft endpoint, seed 0 at advance
5469 / `0xBEB9762F`. One menu opening, one successful theft, then close the
item menu and finish the success dialogue. No intervening actions/time change.

| Boundary | Seed advances | CRT state |
| --- | ---: | --- |
| Before path-action menu | 5469 | 0xBEB9762F |
| Menu opened | 5470 | 0x69EBB836 |
| Soulstone obtained, item menu still open | 5471 | 0x87F3CE21 |
| Success dialogue selected; field control restored | 5472 | 0x956EF860 |

Source-derived prediction; this theft's endpoint is not yet live-verified.

- `NPC_Twn_Cty_1_1_B_TALK_0600_D000` resolves through NPCData to the English
  name **Boy**. Its shop has one item, `ITM_CSM_0440`, **Shadow Soulstone**.
- Stock row `NPC_Twn_Cty_1_1_B_TALK_0600_NPCBUY_01` has ProperSteal=-15.
  At level 2 the difference is 17, meeting STEAL_PROBABILITY_LEVEL_OVER.Init=11;
  STEAL_PROBABILITY_OVER.Init=100 supplies guaranteed success.
- Opening the normal preview consumes one CRT draw through the common action
  timer; see `throne-ambush.md`.
- `FieldCommandWidgetSteal.StealItemExec` always draws
  `RandomIntegerInRange(1,100)`, even at 100% success. It returns IsListClose=false
  when there is no item-triggered story event, retaining the item list.
- This placement has only an ordinary talk event, so
  `GetPlayableEventLabelByGotItems` finds no type-7/type-26 override. After the
  list closes, Steal flow state 1 advances to state 2, which calls DrawEventLabel
  once to choose FC_TOU_STEAL_Success_1 or _2.
- Both success scripts contain one fixed command-42 line and command 10000.
  Both have EncountReset=false. Their cleanup adds no counter pair or new NPC.

The extra third draw is the ordinary success-line lottery. The preceding clothes
theft skipped it because its item-specific story event replaced that dialogue.
