using System;

namespace OgsSgfDownloader.Models
{
    [Serializable] public class White
    {
        public int Id { get; set; }
        public string Username { get; set; }
        public string Country { get; set; }
        public string Icon { get; set; }
        public Ratings Ratings { get; set; }
        public double Ranking { get; set; }
        public bool Professional { get; set; }
        public string UiClass { get; set; }
    }
}