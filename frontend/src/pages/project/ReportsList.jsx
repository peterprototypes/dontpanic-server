import React from 'react';
import useSWR from 'swr';
import { Link as RouterLink, useNavigate, useSearchParams } from 'react-router';
import { DateTime } from "luxon";
import { TableContainer, Tooltip, Typography, TableCell, TableRow, Table, TableHead, TableBody, Checkbox, Paper, Stack } from '@mui/material';
import { DataGrid, useGridApiRef } from '@mui/x-data-grid';

const ReportsList = ({ resolved = false }) => {
  const apiRef = useGridApiRef();
  const navigate = useNavigate();

  const [searchParams] = useSearchParams();
  const projectId = searchParams.get('projectId');

  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });

  let params = new URLSearchParams();
  params.append('cursor', paginationModel.page);
  params.append('resolved', resolved ? 1 : 0);

  if (projectId) {
    params.append('project_id', projectId);
  }

  const { data } = useSWR(`/api/reports?${params.toString()}`);

  if (data?.length === 0) {
    return <NoReports />;
  }

  return (
    <TableContainer>
      <Table size="small">
        <TableHead sx={{ textTransform: 'uppercase' }}>
          <TableRow>
            <TableCell>
              <Checkbox />
            </TableCell>
            <TableCell>#</TableCell>
            <TableCell>Title</TableCell>
            <TableCell>Environment</TableCell>
            <TableCell>Last Seen</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {data?.map((row) => (
            <TableRow key={row.report.project_report_id} onClick={() => navigate(`/report/${row.report.project_report_id}`)}>
              <TableCell>
                <Checkbox />
              </TableCell>
              <TableCell>{row.report.project_report_id}</TableCell>
              <TableCell sx={{ fontWeight: 'bold' }}>{row.report.title}</TableCell>
              <TableCell>{row.env?.name}</TableCell>
              <TableCell>
                <Tooltip title={DateTime.fromISO(row.report.last_seen, { zone: 'UTC' }).toLocaleString(DateTime.DATETIME_FULL)}>
                  <Typography variant="body2" noWrap>{DateTime.fromISO(row.report.last_seen, { zone: 'UTC' }).toRelative()}</Typography>
                </Tooltip>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
};

const NoReports = () => {
  return (
    <Paper sx={{ px: 5, py: 4, backgroundColor: 'accentBackground' }}>
      <Stack spacing={1} alignItems="center" useFlexGap>
        {/* <ProjectIcon sx={{ fontSize: 60 }} /> */}
        <Typography variant="h5" textAlign="center">No Reports Submitted</Typography>
        <Typography variant="body1" textAlign="center" gutterBottom>
          Your application is either bug free or dontpanic library isn&lsquo;t set up correctly to send reports to this server.
        </Typography>
        <Typography variant="body2" textAlign="center">
          To verify reporting is working, add:
          <br />
          <pre>Option::&lt;()&gt;::None.unwrap();</pre>
          after dontpanic initialization and make a test.
        </Typography>
      </Stack>
    </Paper>
  );
};

const MultilineDateTime = ({ value }) => {
  return (
    <>
      <Typography variant="body2" gutterBottom>{value.toLocaleTimeString()}</Typography>
      <Typography variant="body2">{value.toLocaleDateString()}</Typography>
    </>
  );
};

export default ReportsList;