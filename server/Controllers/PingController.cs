using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Server.Models;
using Server.Services;

namespace Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    [Authorize]
    public class PingController(PingRecorder pingRecorder) : ControllerBase
    {
        [HttpPost("")]
        [ProducesResponseType<PingResponse>(StatusCodes.Status200OK)]
        public async Task<PingResponse> Ping()
        {
            DateTime timeRecorded = await pingRecorder.Record();

            return new PingResponse
            {
                TimeRecorded = timeRecorded,
            };
        }
    }
}
