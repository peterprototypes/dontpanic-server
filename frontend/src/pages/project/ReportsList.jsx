import React from 'react';
import useSWR from 'swr';
import { Link as RouterLink, useNavigate } from 'react-router';
import { DateTime } from "luxon";
import { Tooltip } from '@mui/material';
import { DataGrid, GridActionsCellItem } from '@mui/x-data-grid';

const ReportsList = ({ resolved = false }) => {
  const navigate = useNavigate();

  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });

  const { data } = useSWR(`/api/reports?cursor=${paginationModel.page}`);

  console.log(data);

  const columns = React.useMemo(() => [
    { field: 'project_report_id', headerName: '#' },
    { field: 'title', headerName: 'Title', flex: 1 },
    // { field: 'project', headerName: 'Project' },
    { field: 'environment', headerName: 'Environment' },
    {
      field: 'last_seen',
      type: 'dateTime',
      headerName: 'Last Seen',
      minWidth: 160,
      valueGetter: (value) => value && DateTime.fromISO(value, { zone: 'UTC' }).toJSDate()
    }
  ], []);

  return (
    <DataGrid
      rows={[]}
      columns={columns}
      loading={false}
      getRowId={(row) => row.user_id}
      checkboxSelection
      disableRowSelectionOnClick
      onRowClick={({ row }) => navigate(`/report/${row.project_report_id}`)}
      paginationMode="server"
      paginationModel={paginationModel}
      onPaginationModelChange={setPaginationModel}
      rowCount={-1}
    />
  );
};

export default ReportsList;