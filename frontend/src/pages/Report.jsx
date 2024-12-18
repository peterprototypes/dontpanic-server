import useSWR from 'swr';
import { DateTime } from "luxon";
import { useParams, Link as RouterLink, useSearchParams } from 'react-router';
import { Box, Divider, Grid2 as Grid, Link, Typography, Stack, Tooltip, Button, Paper } from '@mui/material';
import { styled } from '@mui/system';

import SideMenu from 'components/SideMenu';
import { BackIcon } from 'components/ConsistentIcons';
import { NextIcon } from '../components/ConsistentIcons';

const Report = () => {
  const { id } = useParams();
  const [searchParams, setSearchParams] = useSearchParams();

  let eventId = searchParams.get('event_id');

  const { data, isLoading, error } = useSWR(`/api/reports/${id}`);
  const { data: event } = useSWR(`/api/reports/${id}/get-event${eventId ? `?event_id=${eventId}` : ''}`);

  if (isLoading) return <Box>Loading...</Box>;

  if (error) return <Box>Error: {error.message}</Box>;

  return (
    <Grid container spacing={2}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9} sx={{ mt: 2 }}>
        <Link component={RouterLink} to={`/reports?project_id=${data.report.project_id}`} color="primary" sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
          <BackIcon />
          Back to reports
        </Link>

        <Stack direction="row" spacing={2} alignItems="center">
          <Typography variant="h6" sx={{ mt: 2 }} color="textSecondary">#{data.report.project_report_id}</Typography>
          <Typography variant="h6" sx={{ mt: 2, fontWeight: '600', fontSize: '15px' }}>{data.report.title}</Typography>
        </Stack>

        <Divider sx={{ my: 2 }} />

        <Stack direction="row" justifyContent="space-between" sx={{ mb: 2 }}>
          <Stack>
            <Typography variant="h6" sx={{ fontSize: '15px' }}>Project</Typography>
            <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">{data.project.name}</Typography>
          </Stack>
          <Stack>
            <Typography variant="h6" sx={{ fontSize: '15px' }}>Environment</Typography>
            <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">{data?.env.name ?? '-'}</Typography>
          </Stack>
          <Stack>
            <Typography variant="h6" sx={{ fontSize: '15px' }}>Last Seen</Typography>
            <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary"><DateTimeDisplay value={data.report.last_seen} /></Typography>
          </Stack>
          <Stack>
            <Typography variant="h6" sx={{ fontSize: '15px' }}>Created</Typography>
            <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary"><DateTimeDisplay value={data.report.created} /></Typography>
          </Stack>
        </Stack>

        <Occurrences occurrences={data.occurrences} maxOccurrences={data.max_occurrences} />

        {event && <Event reportEvent={event} setSearchParams={setSearchParams} />}
      </Grid>
    </Grid>
  );
};

const Event = ({ reportEvent, setSearchParams }) => {
  let data = JSON.parse(reportEvent.event.event_data);

  return (
    <Box sx={{ my: 4 }}>
      <Stack direction="row" spacing={2} alignItems="center" justifyContent="space-between">
        <Typography variant="h6" sx={{ fontWeight: '600', fontSize: '15px' }}>
          Event {reportEvent.event_pos} / {reportEvent.events_count} received at {DateTime.fromISO(reportEvent.event.created, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}
        </Typography>
        <Stack direction="row" spacing={1}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<BackIcon />}
            disabled={!reportEvent.event.prev_event_id}
            onClick={() => setSearchParams({ event_id: reportEvent.event.prev_event_id })}
          >
            Prev
          </Button>
          <Button
            variant="outlined"
            size="small"
            endIcon={<NextIcon />}
            disabled={!reportEvent.event.next_event_id}
            onClick={() => setSearchParams({ event_id: reportEvent.event.next_event_id })}
          >
            Next
          </Button>
        </Stack>
      </Stack>

      <Divider sx={{ my: 2 }} />

      <Stack direction="row" justifyContent="space-between">
        <Stack>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>Location</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">
            {data.loc ? `${data.loc.f}:${data.loc.l}${data.loc.c ? ':' + data.loc.c : ''}` : 'Unknown'}
          </Typography>
        </Stack>
        <Stack>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>Version</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">{data?.ver ?? '-'}</Typography>
        </Stack>
        <Stack>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>OS / Arch</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">
            {data?.os ?? '-'} / {data?.arch ?? '-'}
          </Typography>
        </Stack>
        <Stack>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>Thread Name</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">
            {data?.tname ?? 'Unknown'}
            {data?.tid ? ` / ${data.tid}` : ''}
          </Typography>
        </Stack>
      </Stack>

      <Typography variant="h6" sx={{ fontSize: '14px', mt: 4 }}>Stack Trace</Typography>
      <Code>{data.trace}</Code>

      <Typography variant="h6" sx={{ fontSize: '14px', mt: 4 }}>Log Output</Typography>

      {data.log && <LogMessages log={data.log} />}

      {!data.log && (
        <Box sx={{ mt: 2 }}>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>No Log Messages Recorded</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">
            The dontpanic client library either isn&lsquo;t configured to record log messages or there were no log messages before the panic happened.
          </Typography>
        </Box>
      )}
    </Box>
  );
};

const LogMessages = ({ log }) => {
  return (
    <Paper sx={{ p: 2, mt: 2 }}>
      <Box>
        {log.map((message, i) => (
          <Box key={i} sx={{ mb: 1, fontSize: '13px', fontFamily: 'Consolas, monospace' }}>
            [
            {DateTime.fromMillis(message.ts * 1000, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}
            {' '}
            <Typography component="span" color={message.lvl === 1 ? 'error' : message.lvl === 2 ? 'warning' : message.lvl === 3 ? 'info' : 'textSecondary'}>
              {message.lvl === 1 ? 'ERROR' : message.lvl === 2 ? 'WARN' : message.lvl === 3 ? 'INFO' : message.lvl === 4 ? 'DEBUG' : message.lvl === 5 ? 'TRACE' : ''}
            </Typography>
            {' '}
            {message.mod}
            ]: {message.msg}
          </Box>
        ))}
      </Box>
    </Paper>
  );
};

const DateTimeDisplay = ({ value }) => {
  return (
    <Tooltip title={DateTime.fromISO(value, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}>
      <Box component="span">{DateTime.fromISO(value, { zone: 'UTC' }).toRelative()}</Box>
    </Tooltip>
  );
};

const Occurrences = ({ occurrences, maxOccurrences }) => {

  const getTitle = (day) => {
    return DateTime.fromISO(day.date, { zone: 'UTC' }).toLocaleString(DateTime.DATE_FULL) + ' - ' + day.events_count + ' events';
  };

  return (
    <Box sx={{ display: 'flex', fontSize: '12px' }}>
      <Stack direction="row" sx={{ width: '100%' }} justifyContent="space-between">
        <Stack useFlexGap spacing={0.5} sx={{ textAlign: 'right', pr: '2px' }}>
          <OccuranceWeekday sx={{ height: '14px' }}></OccuranceWeekday>
          <OccuranceWeekday>Mon</OccuranceWeekday>
          <OccuranceWeekday></OccuranceWeekday>
          <OccuranceWeekday></OccuranceWeekday>
          <OccuranceWeekday>Thu</OccuranceWeekday>
          <OccuranceWeekday></OccuranceWeekday>
          <OccuranceWeekday></OccuranceWeekday>
          <OccuranceWeekday>Sun</OccuranceWeekday>
        </Stack>
        {occurrences.map((occurrence, i) => (
          <Stack key={i} useFlexGap spacing={0.5}>
            <OccuranceMonthBox>{occurrence.month_label ?? ' '}</OccuranceMonthBox>
            {occurrence.days.map((day, j) => (
              <Tooltip key={j} title={getTitle(day)}>
                <Day eventsCount={day.events_count} style={{ filter: day.events_count > 0 ? `opacity(${Math.floor((day.events_count / maxOccurrences) * 100)}%)` : null }}>{' '}</Day>
              </Tooltip>
            ))}
          </Stack>
        ))}
      </Stack>
    </Box>
  );
};

const OccuranceWeekday = (props) => (
  <Box component="span" sx={{ height: '9px', display: 'inline-flex', alignItems: 'center', justifyContent: 'end' }} {...props} />
);

const OccuranceMonthBox = (props) => (
  <Box component="span" sx={{ height: '14px', width: '9px' }} {...props} />
);

const Day = styled('span')(({ theme, eventsCount }) => ({
  width: '9px',
  height: '9px',
  border: '1px solid',
  borderColor: eventsCount > 0 ? theme.palette.error.dark : theme.palette.divider,
  backgroundColor: eventsCount > 0 ? theme.palette.error.main : 'transparent',
}));

const Code = styled('pre')(({ theme }) => ({
  padding: theme.spacing(1),
  backgroundColor: theme.palette.background.default,
  borderRadius: theme.shape.borderRadius,
  overflowX: 'auto',
  overflowY: 'auto',
  maxHeight: '600px',
  fontSize: '12px',
  lineHeight: '1.5',
  fontFamily: 'Consolas, monospace',
  color: theme.palette.text.primary,
}));

export default Report;