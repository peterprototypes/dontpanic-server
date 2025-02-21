import useSWR from 'swr';
import { DateTime } from "luxon";
import { useParams, Link as RouterLink } from 'react-router';
import { Box, Divider, Grid2 as Grid, Link, Typography, Stack, Tooltip, Paper } from '@mui/material';
import { BarChart } from '@mui/x-charts/BarChart';
import { styled } from '@mui/system';

import SideMenu from 'components/SideMenu';
import { BackIcon } from 'components/ConsistentIcons';
import LoadingPage from 'components/LoadingPage';

const Report = () => {
  const { id } = useParams();

  const { data, isLoading, error } = useSWR(`/api/reports/${id}`);

  if (isLoading) {
    return (
      <ReportPage><LoadingPage /></ReportPage>
    );
  }

  if (error) {
    return (
      <ReportPage>
        <Link component={RouterLink} to="/reports" color="primary" sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
          <BackIcon />
          Back to reports
        </Link>

        <Typography variant="h6" sx={{ mt: 2 }} color="error">{error.status} - {error.message}</Typography>

        <Divider sx={{ my: 2 }} />
      </ReportPage>
    );
  }

  return (
    <ReportPage>
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
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">{data?.env?.name ?? '-'}</Typography>
        </Stack>
        <Stack>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>Last Seen</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary"><DateTimeDisplay value={data.report.last_seen} /></Typography>
        </Stack>
        <Stack>
          <Typography variant="h6" sx={{ fontSize: '15px' }}>First Appeared</Typography>
          <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary"><DateTimeDisplay value={data.report.created} /></Typography>
        </Stack>
      </Stack>

      <DailyEvents dailyEvents={data.daily_events} />

      <Stack direction="row" spacing={2} sx={{ mt: 4 }}>
        <Box sx={{ width: '100%', borderRadius: 2, backgroundColor: 'background.default' }}>
          <Typography variant="h6" textAlign="center" sx={{ fontSize: '15px', p: 2 }}>Events Received by Version</Typography>
          <BarChart
            dataset={data.version_dataset}
            xAxis={[{ scaleType: 'band', dataKey: 'date' }]}
            series={data.version_names.map((name) => ({ dataKey: name, label: name, stack: 'total', stackOrder: 'appearance' }))}
            height={250}
          />
        </Box>

        <Box sx={{ width: '100%', px: 1, borderRadius: 2, backgroundColor: 'background.default' }}>
          <Typography variant="h6" textAlign="center" sx={{ fontSize: '15px', p: 2 }}>Events Received by Operating System</Typography>
          <BarChart
            dataset={data.os_dataset}
            xAxis={[{ scaleType: 'band', dataKey: 'date' }]}
            series={data.os_names.map((name) => ({ dataKey: name, label: name, stack: 'total', stackOrder: 'appearance' }))}
            height={250}
          />
        </Box>
      </Stack>



      {data.last_event && (
        <>
          <Typography variant="h6" sx={{ fontSize: '14px', mt: 4 }}>Latest Stack Trace</Typography>
          <Code>{data.last_event.backtrace}</Code>

          <Typography variant="h6" sx={{ fontSize: '14px', mt: 4 }}>Latest Log Output</Typography>

          {data.last_event.log && <LogMessages log={JSON.parse(data.last_event.log)} />}

          {!data.last_event.log && <NoLogMessages />}
        </>
      )}
    </ReportPage>
  );
};

const ReportPage = ({ children }) => {
  return (
    <Grid container spacing={2}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9} sx={{ mt: 2 }}>
        <Stack>
          {children}
        </Stack>
      </Grid>
    </Grid>
  );
};

const LogMessages = ({ log }) => {
  if (log.length === 0) {
    return <NoLogMessages />;
  }

  return (
    <Paper sx={{ p: 2, mt: 2 }}>
      <Box sx={{ overflow: 'auto' }}>
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

const NoLogMessages = () => (
  <Box sx={{ mt: 2 }}>
    <Typography variant="h6" sx={{ fontSize: '15px' }}>No Log Messages Recorded</Typography>
    <Typography variant="body2" sx={{ fontWeight: '500', mb: 1 }} color="textSecondary">
      The dontpanic client library either isn&lsquo;t configured to record log messages or there were no log messages before the panic happened.
    </Typography>
  </Box>
);

const DateTimeDisplay = ({ value }) => {
  return (
    <Tooltip title={DateTime.fromISO(value, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}>
      <Box component="span">{DateTime.fromISO(value, { zone: 'UTC' }).toRelative()}</Box>
    </Tooltip>
  );
};

const DailyEvents = ({ dailyEvents }) => {
  const maxEvents = Math.max(...Object.values(dailyEvents));

  const getTitle = (day) => {
    return DateTime.fromISO(day.date, { zone: 'UTC' }).toLocaleString(DateTime.DATE_FULL) + ' - ' + day.events_count + ' events';
  };

  let prev_date = DateTime.now().setZone('UTC').startOf('day');
  let weeks = [];
  let current_week = [];
  let month = null;

  for (var i = 0; i <= 365; i++) {
    let date = DateTime.now().setZone('UTC').minus({ days: i }).startOf('day');

    current_week.push({
      date,
      events_count: dailyEvents[date.toISODate()] ?? 0
    });

    if (prev_date.monthShort != date.monthShort) {
      month = prev_date.monthShort;
    }

    if (date.weekday == 1) {
      weeks.push({
        month_label: month,
        days: current_week.reverse()
      });

      month = null;

      current_week = [];
    }

    prev_date = date;
  }

  return (
    <Box sx={{ display: 'flex', fontSize: '12px', alignSelf: 'center' }}>
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
        {weeks.reverse().map((week, i) => (
          <Stack key={i} useFlexGap spacing={0.5} sx={{ pr: '3px' }}>
            <OccuranceMonthBox>{week.month_label ?? ' '}</OccuranceMonthBox>
            {week.days.map((day, j) => (
              <Tooltip key={j} title={getTitle(day)}>
                <Day eventsCount={day.events_count} style={{ filter: day.events_count > 0 ? `opacity(${Math.floor((day.events_count / maxEvents) * 100)}%)` : null }}>{' '}</Day>
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