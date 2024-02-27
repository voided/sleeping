using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Server.Models;
using Server.Services;

namespace Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    [Authorize]
    public class SleepingController(PingRecorder pingRecorder) : ControllerBase
    {
        [Route("ping")]
        [ProducesResponseType<PingResponse>(StatusCodes.Status200OK)]
        public PingResponse Ping()
        {
            return new PingResponse
            {
                Time = DateTime.UtcNow,
            };
        }

        [HttpPost("record")]
        [ProducesResponseType<RecordResponse>(StatusCodes.Status200OK)]
        public async Task<RecordResponse> Record()
        {
            DateTime timeRecorded = await pingRecorder.Record();

            return new RecordResponse
            {
                TimeRecorded = timeRecorded,
            };
        }
    }
}
