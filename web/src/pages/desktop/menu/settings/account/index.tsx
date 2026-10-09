import { useEffect, useState } from 'react';
import { Alert, Button, Divider } from 'antd';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';

import * as api from '@/api/auth.ts';

import { Logout } from './logout.tsx';

export const Account = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const [username, setUsername] = useState('');
  const [loadFailed, setLoadFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    api.getAccount()
      .then((rsp) => {
        if (cancelled) return;
        if (rsp.code !== 0 || typeof rsp.data?.username !== 'string' || !rsp.data.username) {
          throw new Error('Account unavailable');
        }
        setUsername(rsp.data.username);
      })
      .catch(() => {
        if (!cancelled) setLoadFailed(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function changePassword() {
    navigate('/auth/password');
  }

  return (
    <>
      <div className="text-base">{t('settings.account.title')}</div>
      <Divider className="opacity-50" />

      <div className="flex flex-col space-y-8">
        {loadFailed && (
          <Alert
            type="warning"
            showIcon
            message={t('settings.account.unavailable', {
              defaultValue: 'Account details are unavailable. Close and reopen settings to try again.'
            })}
          />
        )}
        <div className="flex items-center justify-between">
          <span>{t('settings.account.webAccount')}</span>
          <span>{username ? username : '-'}</span>
        </div>

        <div className="flex items-center justify-between">
          <span>{t('settings.account.password')}</span>
          <Button type="primary" onClick={changePassword}>
            {t('settings.account.updateBtn')}
          </Button>
        </div>
      </div>

      <Divider className="opacity-50" />

      <Logout />
    </>
  );
};
