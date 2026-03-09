rule "scoring.touchdown_bonus" {
  priority 10
  lifetime permanent
  on TouchdownScored(team)
  when state.scoring_mode == "touchdown_plus_one"
  before {
    emit AwardPoints(team: team, points: 1)
  }
}
