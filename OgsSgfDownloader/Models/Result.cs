using System;

namespace OgsSgfDownloader.Models
{
    [Serializable] public class Result
    {
        public Related Related { get; set; }
        public Players Players { get; set; }
        public int Id { get; set; }
        public string Name { get; set; }
        public int Creator { get; set; }
        public string Mode { get; set; }
        public string Source { get; set; }
        public int Black { get; set; }
        public int White { get; set; }
        public int Width { get; set; }
        public int Height { get; set; }
        public string Rules { get; set; }
        public bool Ranked { get; set; }
        public int Handicap { get; set; }
        public string Komi { get; set; }
        public string TimeControl { get; set; }
        public int BlackPlayerRank { get; set; }
        public string BlackPlayerRating { get; set; }
        public int WhitePlayerRank { get; set; }
        public string WhitePlayerRating { get; set; }
        public int TimePerMove { get; set; }
        public string TimeControlParameters { get; set; }
        public bool DisableAnalysis { get; set; }
        public object Tournament { get; set; }
        public int TournamentRound { get; set; }
        public object Ladder { get; set; }
        public bool PauseOnWeekends { get; set; }
        public string Outcome { get; set; }
        public bool BlackLost { get; set; }
        public bool WhiteLost { get; set; }
        public bool Annulled { get; set; }
        public DateTime Started { get; set; }
        public DateTime Ended { get; set; }
        public object SgfFilename { get; set; }
        public HistoricalRatings HistoricalRatings { get; set; }
        public bool Rengo { get; set; }
        public object RengoBlackTeam { get; set; }
        public object RengoWhiteTeam { get; set; }
        public bool RengoCasualMode { get; set; }
        public object Flags { get; set; }
    }
}