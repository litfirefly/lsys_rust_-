
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, FormControl, InputLabel, MenuItem, Paper, Select } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { UserSessionContext } from '../../context/session';
import { ConfirmButton } from '../../library/dialog';
import { ClearTextField } from '../../library/input';
import { LoadingButton } from '../../library/loading';
import { BaseTablePage } from '../../library/table_page';
import { ItemTooltip } from '../../library/tips';
import { appList, confirmApp, disableApp } from '../../rest/app';
import { useSearchChange } from '../../utils/hook';
import { showTime } from '../../utils/utils';
const filterStatus = {
    status: [
        { key: 1, val: '待审核' },
        { key: 2, val: '已审核' },
        { key: -1, val: '已禁用' },
    ],
};

export default function SystemUserPage(props) {
    const { userData } = useContext(UserSessionContext)
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        total: 0,
    });

    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 90 }
        },
        {
            field: 'name',
            style: { width: 100 },
            label: '名称',
        },
        {
            field: 'client_id',
            style: { width: 120 },
            label: 'AppID',
        },
        {
            field: 'client_secret',
            style: { width: 120 },
            label: 'AppSecret'
        },
        {
            style: { width: 100 },
            label: '审核状态',
            align: "center",
            render: (row) => {
                let delAction = () => {
                    return disableApp({ appid: row.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.map((item) => {
                            if (item.id == row.id) item.status = -1;
                            return item;
                        })
                        setLoadData({
                            ...loadData,
                            rows: rows
                        })
                        return data;
                    })
                };
                let confirmAction = () => {
                    return confirmApp({ appid: row.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.map((item) => {
                            if (item.id == row.id) {
                                item.status = 2;
                                item.confirm_time = Math.round(new Date().getTime() / 1000);
                            }
                            return item;
                        })
                        setLoadData({
                            ...loadData,
                            rows: rows
                        })
                        return data;
                    })
                }
                if (row.status == 1) {
                    return <Fragment>
                        <ConfirmButton
                            message={`确定审核通过 [${row.name}] 吗?`}
                            onAction={confirmAction}
                            renderButton={(props) => {
                                return <Button {...props} size='small'>
                                    审核通过
                                </Button>
                            }} />
                        <ConfirmButton
                            message={`确定要禁用该应用 [${row.name}] 吗?`}
                            onAction={delAction}
                            renderButton={(props) => {
                                return <Button {...props} size='small'>
                                    禁用
                                </Button>
                            }} />
                    </Fragment>
                } else if (row.status == 2) {
                    return <Fragment>
                        <ItemTooltip title={'审核时间:' + showTime(row.confirm_time, "未知")} placement="top"><span>已审核</span></ItemTooltip>
                        <ConfirmButton
                            message={`确定要禁用该应用 [${row.name}] 吗?`}
                            onAction={delAction}
                            renderButton={(props) => {
                                return <Button {...props} size='small'>
                                    禁用
                                </Button>
                            }} />
                    </Fragment>
                } else {
                    return <span>{filterStatus.status.find((e) => { return e.key == row.status })?.val}</span>
                }
            }
        },
        {
            field: 'add_time',
            style: { width: 180 },
            label: '申请时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
    ];
    const [searchParam, setSearchParam] = useSearchChange({
        status: "",
        client_id: "",
        page: 0,
        page_size: 10,
    });
    const [filterData, setfilterData] = useState({
        status: searchParam.get("status"),
        client_id: searchParam.get("client_id")
    })
    const loadAppData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return appList({
            user_id: parseInt(userData.user_data.user_id),
            status: searchParam.get("status"),
            client_id: searchParam.get("client_id"),
            page: searchParam.get("page")||0,
            page_size: searchParam.get("page_size")||10
        }).then((data) => {
            setLoadData({
                ...loadData,
                ...data,
                loading: false
            })
        })
    }
    useEffect(() => {
        setfilterData({
            ...filterData,
            status: searchParam.get("status"),
            client_id: searchParam.get("client_id"),
        })
        loadAppData()
    }, [searchParam])

    return <Fragment>

        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 110, mr: 1 }} size="small"  >
                <InputLabel id="select-type">审核状态</InputLabel>
                <Select
                    labelId="select-type"
                    id="select-type"
                    label="审核状态"
                    disabled={loadData.loading}
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            status: event.target.value
                        })
                    }}
                    value={filterData.status ?? ''}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        filterStatus.status.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`APPID`}
                    type="text"
                    name="code"
                    value={filterData.client_id}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            client_id: nval
                        })
                    }}
                />
            </FormControl>
            <LoadingButton
                onClick={() => {
                    setSearchParam({
                        ...filterData,
                        page: 0
                    }, loadAppData)
                }}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                loading={loadData.loading}
            >
                过滤
            </LoadingButton>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <BaseTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({
                            page: newPage
                        }, loadAppData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 10}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                            page: 0
                        }, loadAppData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}

